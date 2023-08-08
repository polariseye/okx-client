use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use async_trait::async_trait;
use log::*;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use crate::restful::InstType;
use crate::{Instrument, Trade};
use crate::websocket::{EventResponse, Handler, WebsocketConn};
use crate::websocket::order_book_merge::{OrderBookMergeMgr};
use crate::utils::{from_str, to_str};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum OrderBookSize {
    /// 首次推400档快照数据，以后增量推送，每100毫秒推送一次变化的数据
    Default,
    /// 首次推5档快照数据，以后定量推送，每100毫秒当5档快照数据有变化推送一次5档数据
    Size5,
    /// 首次推1档快照数据，以后定量推送，每10毫秒当1档快照数据有变化推送一次1档数据
    BboTbt,
    /// 首次推400档快照数据，以后增量推送，每10毫秒推送一次变化的数据
    /// 只允许交易手续费等级VIP5及以上的API用户订阅。
    BooksL2Tbt,
    /// 首次推50档快照数据，以后增量推送，每10毫秒推送一次变化的数据
    /// 只允许交易手续费等级VIP4及以上的API用户订阅.
    Books50L2Tbt,
}
impl OrderBookSize {
    pub fn channel(&self) -> String {
        match self {
            OrderBookSize::Default => {
                "books".to_string()
            }
            OrderBookSize::Size5 => {
                "books5".to_string()
            }
            OrderBookSize::BboTbt => {
                "bbo-tbt".to_string()
            }
            OrderBookSize::BooksL2Tbt => {
                "books-l2-tb".to_string()
            }
            OrderBookSize::Books50L2Tbt => {
                "books50-l2-tbt".to_string()
            }
        }
    }

    pub fn from_channel(channel:&str) ->Option<Self> {
        match channel {
            "books" => Some(OrderBookSize::Default),
            "books5" => Some(OrderBookSize::Size5),
            "bbo-tbt" => Some(OrderBookSize::BboTbt),
            "books-l2-tb" => Some(OrderBookSize::BooksL2Tbt),
            "books50-l2-tbt" => Some(OrderBookSize::Books50L2Tbt),
            _ => None
        }
    }
}

#[derive(Clone)]
struct OrderBookSubscribeInfo {
    pub inst_id: String,
    pub size: OrderBookSize
}

pub struct PublicWebsocket {
    conn: OnceCell<Arc<WebsocketConn<PublicWebsocket>>>,
    handler: RwLock<Arc<BTreeMap<String, Arc<Box<dyn PublicHandler>>>>>,
    ticker_subscribed: Mutex<Vec<String>>,
    trade_subscribed: Mutex<Vec<String>>,
    orderbook_subscribed: Mutex<Vec<OrderBookSubscribeInfo>>,
    orderbook_merge_mgr: OrderBookMergeMgr,

    trade_symbol_change_subscribed: Mutex<Vec<InstType>>,
}

impl PublicWebsocket {
    pub async fn start(url: &str) -> Arc<Self> {
        let result = Arc::new(Self {
            conn: OnceCell::new(),
            handler: RwLock::new(Arc::new(BTreeMap::new())),
            ticker_subscribed: Mutex::new(vec![]),
            trade_subscribed: Mutex::new(vec![]),
            orderbook_subscribed: Mutex::new(vec![]),
            orderbook_merge_mgr: OrderBookMergeMgr::new(),
            trade_symbol_change_subscribed: Mutex::new(vec![]),
        });

        let week = Arc::downgrade(&result);
        let conn_obj = WebsocketConn::start(week, url).await;
        let _ = result.conn.set(conn_obj);

        result
    }

    pub fn register(&self, handler: impl PublicHandler+'static){
        let mut writer =  self.handler.write().unwrap();
        let id = handler.id();
        if let Some(_val) = writer.get(&id) {
            panic!("repeated handler register:{}", id.clone());
        }

        let mut cloned:BTreeMap<String, Arc<Box<dyn PublicHandler>>> = writer.as_ref().clone();
        cloned.insert(id, Arc::new(Box::new(handler)));

        *writer = Arc::new(cloned);
    }

    pub fn unregister(&self, id: &str) {
        let mut writer =  self.handler.write().unwrap();
        if writer.get(id).is_none() {
            return;
        }

        let mut cloned:BTreeMap<String, Arc<Box<dyn PublicHandler>>> = writer.as_ref().clone();
        cloned.remove(id);

        *writer = Arc::new(cloned);
    }

    fn handlers(&self) -> Arc<BTreeMap<String, Arc<Box<dyn PublicHandler>>>>{
        self.handler.read().unwrap().clone()
    }

    pub fn conn(&self) -> Arc<WebsocketConn<PublicWebsocket>>{
        self.conn.get().unwrap().clone()
    }

    pub async fn ticker_subscribe(&self, inst_id: &str) {
        {
            let mut writer = self.ticker_subscribed.lock().unwrap();
            if writer.iter().any(|item| *item == inst_id) {
                return;
            }
            writer.push(inst_id.into());
        }

        self.ticker_subscribe_detail(inst_id).await
    }

    async fn ticker_subscribe_detail(&self, inst_id: &str){
        let req = TickerEventArg {
            channel: "tickers".to_string(),
            inst_id: inst_id.to_string(),
        };

        let _ = self.conn().send_request("subscribe", &req).await;
    }

    pub async fn ticker_unsubscribe(&self, inst_id: &str){
        {
            let mut writer = self.ticker_subscribed.lock().unwrap();
            for (index, item) in writer.deref().iter().enumerate() {
                if *item == inst_id {
                    writer.remove(index);
                    break
                }
            }
        }

        self.ticker_unsubscribe_detail(inst_id).await;
    }

    async fn ticker_unsubscribe_detail(&self, inst_id: &str){
        let req = TickerEventArg {
            channel: "tickers".to_string(),
            inst_id: inst_id.to_string(),
        };

        let _ = self.conn().send_request("unsubscribe", &req).await;
    }

    pub async fn trade_subscribe(&self, inst_id: &str) {
        {
            let mut writer = self.trade_subscribed.lock().unwrap();
            if writer.iter().any(|item| *item == inst_id) {
                return;
            }
            writer.push(inst_id.into());
        }

        self.trade_subscribe_detail(inst_id).await
    }

    async fn trade_subscribe_detail(&self, inst_id: &str) {
        let req = TickerEventArg {
            channel: "trades".to_string(),
            inst_id: inst_id.to_string(),
        };

        let _ = self.conn().send_request("subscribe", &req).await;
    }

    pub async fn trade_unsubscribe(&self, inst_id: &str){
        {
            let mut writer = self.trade_subscribed.lock().unwrap();
            for (index, item) in writer.deref().iter().enumerate() {
                if *item == inst_id {
                    writer.remove(index);
                    break
                }
            }
        }

        self.trade_unsubscribe_detail(inst_id).await
    }
    async fn trade_unsubscribe_detail(&self, inst_id: &str){
        let req = TickerEventArg {
            channel: "trades".to_string(),
            inst_id: inst_id.to_string(),
        };

        let _ = self.conn().send_request("unsubscribe", &req).await;
    }

    pub async fn orderbook_subscribe(&self, inst_id: &str, size: OrderBookSize) {
        {
            let mut writer = self.orderbook_subscribed.lock().unwrap();
            if writer.iter().any(|item| item.inst_id == inst_id) {
                return;
            }
            writer.push(OrderBookSubscribeInfo{
                inst_id: inst_id.to_string(),
                size: size.clone(),
            });
        }

      self.orderbook_subscribe_detail(inst_id, size).await
    }

    async fn orderbook_subscribe_detail(&self, inst_id: &str, size: OrderBookSize) {
        let req = TickerEventArg {
            channel: size.channel().to_string(),
            inst_id: inst_id.to_string(),
        };

        let _ = self.conn().send_request("subscribe", &req).await;
        self.orderbook_merge_mgr.add_merge(inst_id, size);
    }

    pub async fn orderbook_unsubscribe(&self, inst_id: &str, size: OrderBookSize){
        {
            let mut writer = self.orderbook_subscribed.lock().unwrap();
            for (index, item) in writer.deref().iter().enumerate() {
                if item.inst_id == inst_id {
                    writer.remove(index);
                    break
                }
            }
        }

        self.orderbook_unsubscribe_detail(inst_id, size).await
    }
    async fn orderbook_unsubscribe_detail(&self, inst_id: &str, size: OrderBookSize){
        let req = TickerEventArg {
            channel: size.channel(),
            inst_id: inst_id.to_string(),
        };

        let _ = self.conn().send_request("unsubscribe", &req).await;
        self.orderbook_merge_mgr.remove_merge(inst_id);

    }

    pub async fn trade_symbol_change_subscribe(&self, inst_type: InstType){
        {
            let mut writer = self.trade_symbol_change_subscribed.lock().unwrap();
            if writer.iter().any(|item| *item == inst_type) {
                return;
            }
            writer.push(inst_type.clone());
        }

        self.trade_symbol_change_subscribe_detail(inst_type).await
    }
    async fn trade_symbol_change_subscribe_detail(&self, inst_type: InstType){
        let req = TradeSymbolChangeSubscribeInfo {
            channel: "instruments".to_string(),
            inst_type,
        };

        let _ = self.conn().send_request("subscribe", &req).await;
    }

    pub async fn trade_symbol_unsubscribe(&self, inst_type: InstType){
        {
            let mut writer = self.trade_symbol_change_subscribed.lock().unwrap();
            for (index, item) in writer.deref().iter().enumerate() {
                if *item == inst_type {
                    writer.remove(index);
                    break
                }
            }
        }

        self.trade_symbol_unsubscribe_detail(inst_type).await
    }
    async fn trade_symbol_unsubscribe_detail(&self, inst_type: InstType){
        let req = TradeSymbolChangeSubscribeInfo {
            channel: "instruments".to_string(),
            inst_type,
        };

        let _ = self.conn().send_request("unsubscribe", &req).await;
    }

    pub fn orderbook_merge(&self) -> &OrderBookMergeMgr {
        &self.orderbook_merge_mgr
    }
}

#[derive(Serialize, Deserialize)]
pub struct TradeSymbolChangeSubscribeInfo{
    channel: String,
    #[serde(rename="instType")]
    inst_type: InstType,
}
#[async_trait]
impl Handler for PublicWebsocket {
    async fn on_connected(&self) {
        // 需要处理订阅不会生效的逻辑.
        let ticker_subscribed = {
            self.ticker_subscribed.lock().unwrap().clone()
        };
        for item in ticker_subscribed {
            self.ticker_subscribe_detail(&item).await;
        }

        let trade_subscribed = {
          self.trade_subscribed.lock().unwrap().clone()
        };
        for item in trade_subscribed {
            self.trade_subscribe_detail(&item).await;
        }

        let orderbook_subscribed = {
            self.orderbook_subscribed.lock().unwrap().clone()
        };
        for item in orderbook_subscribed {
            self.orderbook_subscribe_detail(&item.inst_id, item.size).await;
        }

        let trade_symbol_change_subscribed = {
            let lock_val = self.trade_symbol_change_subscribed.lock().unwrap();
            let mut result = vec![];
            for item in lock_val.iter() {
                result.push(item.clone());
            }
            result
        };
        for item in trade_symbol_change_subscribed {
            self.trade_symbol_change_subscribe(item).await;
        }

        for item in self.handlers().values() {
            item.on_connected().await;
        }
    }

    async fn on_disconnected(&self) {
        for item in self.handlers().values() {
            item.on_disconnected().await;
        }
    }

    async fn handle_response(&self, resp: EventResponse) {
        let handlers = self.handlers();
        for item in  handlers.values() {
            item.handle_response(&resp).await;
        }

        if resp.code != "0" {
            error!("receive error. code:{} msg:{}", &resp.code, &resp.msg);
            return;
        }

        let channel ;
        match resp.channel() {
            Some(val) => channel = val,
            None => {
                return;
            }
        }

        match channel.as_str() {
            "tickers" => {
                let arg;
                if let Some(val) = &resp.arg {
                    match serde_json::from_value(val.clone()) {
                        Ok(val) => {
                           arg = val;
                        },
                        Err(err) => {
                            error!("unmarshal ticker arg error:{}", err.to_string());
                            return;
                        }
                    }
                } else {
                    error!("receive ticker event. but have no arg");
                    return;
                }
                if let Some(data) = resp.data {
                    match serde_json::from_value(data) {
                        Ok(ticker_data) => {
                            for item in handlers.values() {
                                item.ticker_event(&arg,&ticker_data).await;
                            }
                        },
                        Err(err) => {
                            error!("unmarshal ticker data error:{}", err.to_string());
                        }
                    }
                } else {
                    debug!("receive ticker event. but have no data");
                }
            },
            "trades" => {
                let arg;
                if let Some(val) = &resp.arg {
                    match serde_json::from_value(val.clone()) {
                        Ok(val) => {
                            arg = val;
                        },
                        Err(err) => {
                            error!("unmarshal trades arg error:{}", err.to_string());
                            return;
                        }
                    }
                } else {
                    error!("receive trades event. but have no arg");
                    return;
                }

                if let Some(data) = resp.data {
                    match serde_json::from_value(data) {
                        Ok(traded_data) => {
                            for item in handlers.values() {
                                item.trade_event(&arg,&traded_data).await;
                            }
                        },
                        Err(err) => {
                            error!("unmarshal trade data error:{}", err.to_string());
                        }
                    }
                } else {
                    debug!("receive trade event. but have no data");
                }
            },
            "books"|"books5"|"bbo-tbt"|"books-l2-tb"|"books50-l2-tbt" => {
                let orderbook_size ;
                if let Some(size) = OrderBookSize::from_channel(&channel) {
                    orderbook_size = size;
                } else {
                    error!("convert orderbook size error");
                    return;
                }

                let orderbook_type ;
                if let Some(action) = OrderBookType::from_action(&resp.action) {
                    orderbook_type = action;
                } else {
                    error!("convert orderbook type error");
                    return;
                }

                let arg;
                if let Some(val) = &resp.arg {
                    match serde_json::from_value(val.clone()) {
                        Ok(val) => {
                            arg = val;
                        },
                        Err(err) => {
                            error!("unmarshal orderbook arg error:{}", err.to_string());
                            return;
                        }
                    }
                } else {
                    error!("receive orderbook event. but have no arg");
                    return;
                }

                if let Some(data) = resp.data {
                    match serde_json::from_value(data) {
                        Ok(orderbook_data) => {
                            for item in handlers.values() {
                                item.orderbook_event(&arg,orderbook_type, orderbook_size, &orderbook_data).await;
                            }

                            self.orderbook_merge_mgr.orderbook_event(&arg, orderbook_type, orderbook_size, &orderbook_data).await;
                        }
                        Err(err) => {
                            error!("unmarshal trade data error:{}", err.to_string());
                        }
                    }
                } else {
                    debug!("receive orderbook event. but have no data");
                }
            }
            "instruments" => {
                if let Some(data) = resp.data {
                    match serde_json::from_value(data) {
                        Ok(instrument_data) => {
                            for item in handlers.values() {
                                item.instrument_event(&instrument_data).await;
                            }
                        }
                        Err(err) => {
                            error!("unmarshal instrument data error:{}", err.to_string());
                        }
                    }
                } else {
                    debug!("receive trade event. but have no data");
                }
            }
            _ => {

            }
        }
    }
}

#[async_trait]
#[allow(unused)]
pub trait PublicHandler: Send + Sync {
    fn id(&self) -> String;
    async fn on_connected(&self){}
    async fn on_disconnected(&self){}

    /// 行情事件
    async fn ticker_event(&self, arg: &TickerEventArg, events: &Vec<TickerEvent>){}
    async fn trade_event(&self, arg:&TradeEventArg, events: &Vec<TradeEvent>){}
    async fn orderbook_event(&self, arg: &OrderBookEventArg, order_book_type: OrderBookType, size: OrderBookSize, events: &Vec<OrderBookEvent>){}
    async fn instrument_event(&self, events: &Vec<Instrument>){}

    async fn handle_response(&self, resp: &EventResponse){}
}

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
pub enum OrderBookType {
    /// 全量
    #[serde(rename="snapshot")]
    Snapshot,
    /// 增量
    #[serde(rename="update")]
    Update
}

impl OrderBookType {
    pub fn from_action(action: &str) -> Option<OrderBookType> {
        match action {
            "snapshot" => Some(OrderBookType::Snapshot),
            "update" => Some(OrderBookType::Update),
            _ => None
        }
    }
}

#[derive(Clone,Serialize, Deserialize)]
pub struct TickerEventArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub inst_id: String,
}

#[derive(Clone,Serialize, Deserialize)]
pub struct TickerEvent {
    #[serde(rename = "instType")]
    pub inst_type: InstType,
    #[serde(rename = "instId")]
    pub inst_id: String,
    pub last: String,
    #[serde(rename = "lastSz")]
    pub last_sz: String,
    #[serde(rename = "askPx")]
    pub ask_px: String,
    #[serde(rename = "askSz")]
    pub ask_sz: String,
    #[serde(rename = "bidPx")]
    pub bid_px: String,
    #[serde(rename = "bidSz")]
    pub bid_sz: String,
    pub open24h: String,
    pub high24h: String,
    pub low24h: String,
    #[serde(rename = "volCcy24h")]
    pub vol_ccy24h: String,
    pub vol24h: String,
    #[serde(rename = "sodUtc0")]
    pub sod_utc0: String,
    #[serde(rename = "sodUtc8")]
    pub sod_utc8: String,
    #[serde(serialize_with="to_str",deserialize_with="from_str")]
    pub ts: i64,
}

pub type TradeEventArg = TickerEventArg;
pub type TradeEvent = Trade;

pub type OrderBookEventArg = TickerEventArg;

#[derive(Clone,Serialize, Deserialize)]
pub struct OrderBookEvent {
    pub asks: Vec<Vec<String>>,
    pub bids: Vec<Vec<String>>,
    #[serde(serialize_with="to_str",deserialize_with="from_str")]
    pub ts: i64,
    pub checksum: i64,
    #[serde(rename = "prevSeqId")]
    pub prev_seq_id: i64,
    #[serde(rename = "seqId")]
    pub seq_id: i64,
}