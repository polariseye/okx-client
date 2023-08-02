use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use async_trait::async_trait;
use http::Method;
use log::{debug, error};
use once_cell::sync::OnceCell;
use ring::hmac;
use serde::{Deserialize, Serialize};
use crate::restful::models::InstType;
use crate::utils;
use crate::websocket::conn::{EventResponse, Handler, WebsocketConn, WebsocketRequest};

pub struct AccountWebsocket {
    api_key: String,
    secret_key: String,
    passphrase: String,
    conn: OnceCell<Arc<WebsocketConn<AccountWebsocket>>>,
    handler: RwLock<Arc<BTreeMap<String, Arc<Box<dyn AccountHandler>>>>>,
    is_account_subscribed: AtomicBool,
    order_subscribed: Mutex<Vec<InstType>>,
}

impl AccountWebsocket {
    pub async fn start(api_key: &str, secret_key: &str, passphrase: &str, url: &str) -> Arc<Self> {
        let result = Arc::new(Self {
            api_key: api_key.to_string(),
            secret_key: secret_key.to_string(),
            passphrase: passphrase.to_string(),
            conn: OnceCell::new(),
            handler: RwLock::new(Arc::new(BTreeMap::new())),
            is_account_subscribed: Default::default(),
            order_subscribed: Mutex::new(vec![]),
        });

        let week = Arc::downgrade(&result);
        let conn_obj = WebsocketConn::start(week, url).await;
        let _ = result.conn.set(conn_obj);

        result
    }

    pub fn conn(&self) -> Arc<WebsocketConn<AccountWebsocket>>{
        self.conn.get().unwrap().clone()
    }

    pub fn register(&self, handler: impl AccountHandler+'static){
        let mut writer =  self.handler.write().unwrap();
        let id = handler.id();
        if let Some(_val) = writer.get(&id) {
            panic!("repeated handler register:{}", id.clone());
        }

        let mut cloned:BTreeMap<String, Arc<Box<dyn AccountHandler>>> = writer.as_ref().clone();
        cloned.insert(id, Arc::new(Box::new(handler)));

        *writer = Arc::new(cloned);
    }

    pub fn unregister(&self, id: &str) {
        let mut writer =  self.handler.write().unwrap();
        if writer.get(id).is_none() {
            return;
        }

        let mut cloned:BTreeMap<String, Arc<Box<dyn AccountHandler>>> = writer.as_ref().clone();
        cloned.remove(id);

        *writer = Arc::new(cloned);
    }

    fn handlers(&self) -> Arc<BTreeMap<String, Arc<Box<dyn AccountHandler>>>>{
        self.handler.read().unwrap().clone()
    }

    fn get_timestamp(&self) -> i64 {
        chrono::Utc::now().timestamp()
    }

    fn sign(&self, time_stamp: String, method: http::Method, data: &str) -> String {
        let message = format!("{}{}{}", time_stamp, method.as_str(), data);
        let hmac_key = ring::hmac::Key::new(hmac::HMAC_SHA256, &self.secret_key.as_bytes());
        let result = ring::hmac::sign(&hmac_key, &message.as_bytes());
        base64::encode(result)
    }

    async fn login(&self) {
        #[derive(Serialize)]
        struct Request {
            #[serde(rename = "apiKey")]
            pub api_key: String,
            pub passphrase: String,
            pub timestamp: String,
            pub sign: String,
        }

        let mut req = Request {
            api_key: self.api_key.to_string(),
            passphrase: self.passphrase.to_string(),
            timestamp: self.get_timestamp().to_string(),
            sign: "".to_string(),
        };
        req.sign = self.sign(req.timestamp.clone(), Method::GET, "/users/self/verify");

        let _ = self.conn().send_request("login", req).await;
    }

    pub async fn account_subscribe(&self) {
        if !self.is_account_subscribed.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            return;
        }

        self.account_subscribe_detail().await
    }
    async fn account_subscribe_detail(&self) {
        #[derive(Serialize)]
        struct Request {
            pub channel: String,
            #[serde(rename="extraParams")]
            pub extra_params: String,
        }

        let req = Request {
            channel: "account".to_string(),
            extra_params: "{\"updateInterval\":0}".into(),
        };

        let _ = self.conn().send_request("subscribe", &req).await;
    }

    pub async fn account_unsubscribe(&self) {
        if !self.is_account_subscribed.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            return;
        }

        self.account_unsubscribe_detail().await;
    }
    async fn account_unsubscribe_detail(&self) {
        #[derive(Serialize)]
        struct Request {
            pub channel: String,
        }

        let req = Request {
            channel: "account".to_string(),
        };

        let _ = self.conn().send_request("unsubscribe", &req).await;
    }

    pub async fn order_subscribe(&self, inst_type: InstType) {
        {
            let mut writer = self.order_subscribed.lock().unwrap();
            if writer.iter().any(|item| *item == inst_type) {
                return;
            }
            writer.push(inst_type.clone());
        }

        self.order_subscribe_detail(inst_type).await
    }
    async fn order_subscribe_detail(&self, inst_type: InstType) {
        let req = OrderSubscribeArg {
            channel: "orders".to_string(),
            inst_type,
            inst_family: None,
            inst_id: None,
        };

        let _ = self.conn().send_request("subscribe", &req).await;
    }

    pub async fn order_unsubscribe(&self, inst_type: InstType){
        {
            let mut writer = self.order_subscribed.lock().unwrap();
            for (index, item) in writer.deref().iter().enumerate() {
                if *item == inst_type {
                    writer.remove(index);
                    break
                }
            }
        }

        self.order_unsubscribe_detail(inst_type).await
    }
    async fn order_unsubscribe_detail(&self, inst_type: InstType){
        {
            let mut writer = self.order_subscribed.lock().unwrap();
            for (index, item) in writer.deref().iter().enumerate() {
                if *item == inst_type {
                    writer.remove(index);
                    break
                }
            }
        }

        let req = OrderSubscribeArg {
            channel: "orders".to_string(),
            inst_type,
            inst_family: None,
            inst_id: None,
        };

        let _ = self.conn().send_request("unsubscribe", &req).await;
    }
}

#[async_trait]
impl Handler for AccountWebsocket {
    async fn on_connected(&self) {
        println!("connected");
        self.login().await;

        if self.is_account_subscribed.load(Ordering::SeqCst){
            self.account_subscribe_detail().await;
        }

        let order_subscribe_list = {self.order_subscribed.lock().unwrap().clone()};
        for item in order_subscribe_list {
            self.order_subscribe_detail(item).await;
        }

        for item in self.handlers().values() {
            item.on_connected().await;
        }
    }

    async fn on_disconnected(&self) {
        println!("disconnected");
        for item in self.handlers().values() {
            item.on_disconnected().await;
        }
    }

    async fn handle_response(&self, resp: EventResponse) {
        let handlers = self.handlers();
        for item in  handlers.values() {
            item.handle_response(&resp).await;
        }

        debug!("receive. code:{} msg:{} action:{}", &resp.code, &resp.msg, &resp.action);
        if resp.code != "0" {
            error!("receive error. code:{} msg:{}", &resp.code, &resp.msg);
            return;
        }
        let channel;
        if let Some(val) = resp.channel() {
            channel = val;
        } else {
            return;
        }
        match channel.as_str() {
            "account" => {
                if let Some(data) = resp.data {
                    match serde_json::from_value(data) {
                        Ok(event_data) => {
                            for item in handlers.values() {
                                item.account_event(&event_data).await;
                            }
                        }
                        Err(err) => {
                            error!("unmarshal account data error:{}", err.to_string());
                        }
                    }
                } else {
                    debug!("receive account event. but have no data");
                }
            },
            "orders" => {
                if let Some(data) = resp.data {
                    match serde_json::from_value(data) {
                        Ok(event_data) => {
                            for item in handlers.values() {
                                item.order_event(&event_data).await;
                            }
                        }
                        Err(err) => {
                            error!("unmarshal order data error:{}", err.to_string());
                        }
                    }
                } else {
                    debug!("receive order event. but have no data");
                }
            },
            _ => {
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountEventArg {
    pub channel: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ccy: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub uid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountAssetItemEvent {
    #[serde(rename = "availBal")]
    pub avail_bal: String,
    #[serde(rename = "availEq")]
    pub avail_eq: String,
    pub ccy: String,
    #[serde(rename = "cashBal")]
    pub cash_bal: String,
    #[serde(rename = "uTime")]
    pub u_time: String,
    #[serde(rename = "disEq")]
    pub dis_eq: String,
    pub eq: String,
    #[serde(rename = "eqUsd")]
    pub eq_usd: String,
    #[serde(rename = "frozenBal")]
    pub frozen_bal: String,
    pub interest: String,
    #[serde(rename = "isoEq")]
    pub iso_eq: String,
    pub liab: String,
    #[serde(rename = "maxLoan")]
    pub max_loan: String,
    #[serde(rename = "mgnRatio")]
    pub mgn_ratio: String,
    #[serde(rename = "notionalLever")]
    pub notional_lever: String,
    #[serde(rename = "ordFrozen")]
    pub ord_frozen: String,
    pub upl: String,
    #[serde(rename = "uplLiab")]
    pub upl_liab: String,
    #[serde(rename = "crossLiab")]
    pub cross_liab: String,
    #[serde(rename = "isoLiab")]
    pub iso_liab: String,
    #[serde(rename = "coinUsdPrice")]
    pub coin_usd_price: String,
    #[serde(rename = "stgyEq")]
    pub stgy_eq: String,
    #[serde(rename = "spotInUseAmt")]
    pub spot_in_use_amt: String,
    #[serde(rename = "isoUpl")]
    pub iso_upl: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountEvent {
    #[serde(rename = "uTime")]
    pub u_time: String,
    #[serde(rename = "totalEq")]
    pub total_eq: String,
    #[serde(rename = "isoEq")]
    pub iso_eq: String,
    #[serde(rename = "adjEq")]
    pub adj_eq: String,
    #[serde(rename = "ordFroz")]
    pub ord_froz: String,
    pub imr: String,
    pub mmr: String,
    #[serde(rename = "notionalUsd")]
    pub notional_usd: String,
    #[serde(rename = "mgnRatio")]
    pub mgn_ratio: String,
    pub details: Vec<AccountAssetItemEvent>,
}

#[derive(Serialize, Deserialize)]
pub struct OrderSubscribeArg {
    pub channel: String,
    #[serde(rename = "instType")]
    pub inst_type: InstType,
    #[serde(rename = "instFamily", skip_serializing_if = "Option::is_none")]
    pub inst_family: Option<String>,
    #[serde(rename = "instId", skip_serializing_if = "Option::is_none")]
    pub inst_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct OrderEvent {
    #[serde(rename = "accFillSz")]
    pub acc_fill_sz: String,
    #[serde(rename = "amendResult")]
    pub amend_result: String,
    #[serde(rename = "avgPx")]
    pub avg_px: String,
    #[serde(rename = "cTime")]
    pub c_time: String,
    pub category: String,
    pub ccy: String,
    #[serde(rename = "clOrdId")]
    pub cl_ord_id: String,
    pub code: String,
    #[serde(rename = "execType")]
    pub exec_type: String,
    pub fee: String,
    #[serde(rename = "feeCcy")]
    pub fee_ccy: String,
    #[serde(rename = "fillFee")]
    pub fill_fee: String,
    #[serde(rename = "fillFeeCcy")]
    pub fill_fee_ccy: String,
    #[serde(rename = "fillNotionalUsd")]
    pub fill_notional_usd: String,
    #[serde(rename = "fillPx")]
    pub fill_px: String,
    #[serde(rename = "fillSz")]
    pub fill_sz: String,
    #[serde(rename = "fillPnl")]
    pub fill_pnl: String,
    #[serde(rename = "fillTime")]
    pub fill_time: String,
    #[serde(rename = "instId")]
    pub inst_id: String,
    #[serde(rename = "instType")]
    pub inst_type: String,
    pub lever: String,
    pub msg: String,
    #[serde(rename = "notionalUsd")]
    pub notional_usd: String,
    #[serde(rename = "ordId")]
    pub ord_id: String,
    #[serde(rename = "ordType")]
    pub ord_type: String,
    pub pnl: String,
    #[serde(rename = "posSide")]
    pub pos_side: String,
    pub px: String,
    pub rebate: String,
    #[serde(rename = "rebateCcy")]
    pub rebate_ccy: String,
    #[serde(rename = "reduceOnly")]
    pub reduce_only: String,
    #[serde(rename = "reqId")]
    pub req_id: String,
    pub side: String,
    #[serde(rename = "attachAlgoClOrdId")]
    pub attach_algo_cl_ord_id: String,
    #[serde(rename = "slOrdPx")]
    pub sl_ord_px: String,
    #[serde(rename = "slTriggerPx")]
    pub sl_trigger_px: String,
    #[serde(rename = "slTriggerPxType")]
    pub sl_trigger_px_type: String,
    pub source: String,
    pub state: String,
    #[serde(rename = "stpId")]
    pub stp_id: String,
    #[serde(rename = "stpMode")]
    pub stp_mode: String,
    pub sz: String,
    pub tag: String,
    #[serde(rename = "tdMode")]
    pub td_mode: String,
    #[serde(rename = "tgtCcy")]
    pub tgt_ccy: String,
    #[serde(rename = "tpOrdPx")]
    pub tp_ord_px: String,
    #[serde(rename = "tpTriggerPx")]
    pub tp_trigger_px: String,
    #[serde(rename = "tpTriggerPxType")]
    pub tp_trigger_px_type: String,
    #[serde(rename = "tradeId")]
    pub trade_id: String,
    #[serde(rename = "quickMgnType")]
    pub quick_mgn_type: String,
    #[serde(rename = "algoClOrdId")]
    pub algo_cl_ord_id: String,
    #[serde(rename = "algoId")]
    pub algo_id: String,
    #[serde(rename = "amendSource")]
    pub amend_source: String,
    #[serde(rename = "cancelSource")]
    pub cancel_source: String,
    #[serde(rename = "uTime")]
    pub u_time: String,
}

#[async_trait]
#[allow(unused)]
pub trait AccountHandler: Send + Sync {
    fn id(&self) -> String;
    async fn account_event(&self, events: &Vec<AccountEvent>){}
    async fn order_event(&self, events: &Vec<OrderEvent>){}
    async fn on_connected(&self){}
    async fn on_disconnected(&self){}
    async fn handle_response(&self, resp: &EventResponse){}
}