use crate::websocket::{OrderBookEvent, OrderBookEventArg, OrderBookSize, OrderBookType, PublicHandler};
use log::error;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct OrderBookItem {
    pub price: Decimal,
    pub amount: Decimal,
    pub order_count: u32,
}

pub struct OrderBookMerge {
    /// 卖方深度
    pub asks: BTreeMap<Decimal, OrderBookItem>,
    /// 买方深度
    pub bids: BTreeMap<Decimal, OrderBookItem>,
    pub seq_id: i64,
    pub size: OrderBookSize,
    pub inst_id: String,
}

impl OrderBookMerge {
    pub fn new(inst_id: &str, size: OrderBookSize) -> Self {
        Self {
            asks: BTreeMap::new(),
            bids: BTreeMap::new(),
            seq_id: -1,
            size,
            inst_id: inst_id.to_string(),
        }
    }

    pub fn handle_orderbook_event(&mut self, event: &OrderBookEvent) -> bool {
        if event.prev_seq_id == -1 {
            self.clear();
        } else if self.seq_id != event.prev_seq_id {
            error!("receive wrong seq id .may lost some message");
            return false;
        }

        self.seq_id = event.seq_id;
        if event.bids.is_empty() && event.asks.is_empty() {
            return false;
        }
        if self.size == OrderBookSize::Size5 || self.size == OrderBookSize::BboTbt {
            // 全量推送
            self.clear();
        }

        for item in &event.bids {
            match OrderBookItem::new(item) {
                Ok(val) => {
                    if val.amount.is_zero() {
                        self.bids.remove(&val.price);
                    } else {
                        self.bids.insert(val.price.clone(), val);
                    }
                }
                Err(err) => {
                    error!("handle bids orderbook item error,{}", err.to_string());
                }
            }
        }

        for item in &event.asks {
            match OrderBookItem::new(item) {
                Ok(val) => {
                    if val.amount.is_zero() {
                        self.asks.remove(&val.price);
                    } else {
                        self.asks.insert(val.price.clone(), val);
                    }
                }
                Err(err) => {
                    error!("handle asks orderbook item error,{}", err.to_string());
                }
            }
        }

        true
    }

    pub fn clear(&mut self) {
        self.asks.clear();
        self.bids.clear();
    }

    pub fn get_order_book(&self) -> OrderBook {
        let mut asks = vec![];
        for item in &self.asks {
            asks.push(item.1.clone());
        }

        let mut bids = vec![];
        for item in self.asks.iter().rev() {
            bids.push(item.1.clone());
        }

        return OrderBook {
            asks,
            bids,
            seq_id: self.seq_id,
        }
    }
}

impl OrderBookItem {
    pub fn new(data: &[String]) -> anyhow::Result<Self> {
        let price = Decimal::from_str(&data[0]).map_err(|err| anyhow::anyhow!(err))?;
        let amount = Decimal::from_str(&data[1]).map_err(|err| anyhow::anyhow!(err))?;
        let order_count = u32::from_str(&data[3]).map_err(|err| anyhow::anyhow!(err))?;
        Ok(Self {
            price,
            amount,
            order_count,
        })
    }
}

pub struct OrderBook {
    pub seq_id: i64,

    /// 卖方深度
    pub asks: Vec<OrderBookItem>,
    /// 买方深度
    pub bids: Vec<OrderBookItem>,
}

pub struct OrderBookMergeMgr{
    handler: RwLock<Arc< HashMap<String, Arc<Box<dyn OrderBookMergeHandler>>>>>,
    merges: RwLock<HashMap<String, Arc<RwLock<OrderBookMerge>>>>
}

impl OrderBookMergeMgr {
    pub fn new() -> Self {
        Self{
            handler: RwLock::new(Arc::new(HashMap::new())),
            merges: RwLock::new(HashMap::new()),
        }
    }
    pub fn register(&self, handler: impl OrderBookMergeHandler+'static){
        let mut writer =  self.handler.write().unwrap();
        let id = handler.id();
        if let Some(_val) = writer.get(&id) {
            panic!("repeated handler register:{}", id.clone());
        }

        let mut cloned:HashMap<String, Arc<Box<dyn OrderBookMergeHandler>>> = writer.as_ref().clone();

        cloned.insert(id, Arc::new(Box::new(handler)));

        *writer = Arc::new(cloned);
    }

    pub fn unregister(&self, id: &str) {
        let mut writer =  self.handler.write().unwrap();
        if writer.get(id).is_none() {
            return;
        }

        let mut cloned:HashMap<String, Arc<Box<dyn OrderBookMergeHandler>>> = writer.as_ref().clone();
        cloned.remove(id);

        *writer = Arc::new(cloned);
    }

    fn handlers(&self) -> Arc<HashMap<String, Arc<Box<dyn OrderBookMergeHandler>>>>{
        self.handler.read().unwrap().clone()
    }

    pub fn add_merge(&self, inst_id: &str, orderbook_size: OrderBookSize) -> Arc<RwLock<OrderBookMerge>> {
        let mut writer = self.merges.write().unwrap();
        if let Some(val) = writer.get(inst_id){
            return val.clone();
        }

        let result = Arc::new(RwLock::new(OrderBookMerge::new(inst_id, orderbook_size)));
        writer.insert(inst_id.to_string(), result.clone());

        result
    }

    pub fn remove_merge(&self, inst_id: &str) {
        let mut writer = self.merges.write().unwrap();
        writer.remove(inst_id);
    }

    pub fn get_merge(&self, inst_id: &str) -> Option<Arc<RwLock<OrderBookMerge>>> {
        let reader = self.merges.read().unwrap();
        match reader.get(inst_id) {
            Some(val) => Some(val.clone()),
            None => None,
        }
    }
}

#[async_trait]
impl PublicHandler for OrderBookMergeMgr {
    fn id(&self) -> String {
        "order_book_merge".to_string()
    }

    async fn orderbook_event(&self, arg: &OrderBookEventArg, _order_book_type: OrderBookType, _size: OrderBookSize, events: &Vec<OrderBookEvent>) {
        if let Some(merge) = self.get_merge(&arg.inst_id) {
            let handlers = self.handlers();
            {
                let mut writer = merge.write().unwrap();
                let mut is_changed= false;
                for item in events {
                    if writer.handle_orderbook_event(item) {
                        is_changed = true;
                    }
                }

                if is_changed == false {
                    return;
                }
                if !handlers.is_empty() {
                    let orderbook = writer.get_order_book();
                    tokio::spawn(async move{
                       for item in handlers.values() {
                           item.on_orderbook_update(&orderbook).await;
                       }
                    });
                }
            }
        }
    }
}

#[async_trait]
pub trait OrderBookMergeHandler: Send + Sync {
    fn id(&self) -> String;

    async fn on_orderbook_update(&self, orderbook:&OrderBook);
}