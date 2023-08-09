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
use crate::{ExecuteType, OrderState, OrderType, PositionSide, StopMode, TpTriggerPxType, TradeMode, TradeSide};
use crate::restful::InstType;
use crate::utils::{from_str, to_str};

use crate::websocket::conn::{EventResponse, Handler, WebsocketConn};

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

    async fn init_when_finish_auth(&self){
        if self.is_account_subscribed.load(Ordering::SeqCst){
            self.account_subscribe_detail().await;
        }

        let order_subscribe_list = {self.order_subscribed.lock().unwrap().clone()};
        for item in order_subscribe_list {
            self.order_subscribe_detail(item).await;
        }
    }
}

#[async_trait]
impl Handler for AccountWebsocket {
    async fn on_connected(&self) {
        println!("connected");
        self.login().await;

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

        debug!("receive. event:{} code:{} msg:{} action:{}", &resp.event, &resp.code, &resp.msg, &resp.action);
        match resp.event.as_str() {
            "login" => {
                if resp.code == "0" {
                    self.init_when_finish_auth().await;

                    for item in handlers.values() {
                        item.on_finish_auth().await;
                    }
                } else {
                    error!("login fail received error. code:{} msg:{}", &resp.code, &resp.msg);
                }

                return;
            },
            _ => {

            }
        }

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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AccountEventArg {
    pub channel: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ccy: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub uid: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AccountAssetItemEvent {
    #[serde(rename = "availBal")]
    pub avail_bal: String,
    #[serde(rename = "availEq")]
    pub avail_eq: String,
    pub ccy: String,
    #[serde(rename = "cashBal")]
    pub cash_bal: String,
    #[serde(rename = "uTime", serialize_with="to_str",deserialize_with="from_str")]
    pub u_time: i64,
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AccountEvent {
    #[serde(rename = "uTime", serialize_with="to_str",deserialize_with="from_str")]
    pub u_time: i64,
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

#[derive(Clone, Serialize, Deserialize)]
pub struct OrderSubscribeArg {
    pub channel: String,
    #[serde(rename = "instType")]
    pub inst_type: InstType,
    #[serde(rename = "instFamily", skip_serializing_if = "Option::is_none")]
    pub inst_family: Option<String>,
    #[serde(rename = "instId", skip_serializing_if = "Option::is_none")]
    pub inst_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OrderEvent {
    /// 产品类型
    #[serde(rename = "instType")]
    pub inst_type: InstType,
    /// 产品ID
    #[serde(rename = "instId")]
    pub inst_id: String,
    /// 保证金币种，仅适用于单币种保证金账户下的全仓币币杠杆订单
    pub ccy: String,
    /// 订单ID
    #[serde(rename = "ordId")]
    pub ord_id: String,
    /// 由用户设置的订单ID来识别您的订单
    #[serde(rename = "clOrdId")]
    pub cl_ord_id: String,
    /// 订单标签
    pub tag: String,
    /// 委托价格
    pub px: String,
    /// 原始委托数量，币币/币币杠杆，以币为单位；交割/永续/期权 ，以张为单位
    pub sz: String,
    /// 委托单预估美元价值
    #[serde(rename = "notionalUsd")]
    pub notional_usd: String,
    /// 订单类型
    #[serde(rename = "ordType")]
    pub ord_type: OrderType,
    /// 订单方向，buy sell
    pub side: TradeSide,
    /// 持仓方向
    /// long：开平仓模式开多
    /// short：开平仓模式开空
    /// net：买卖模式
    #[serde(rename = "posSide")]
    pub pos_side: PositionSide,
    /// 交易模式
    /// 保证金模式 isolated：逐仓 cross：全仓
    /// 非保证金模式 cash：现金
    #[serde(rename = "tdMode")]
    pub td_mode: TradeMode,
    /// 市价单委托数量sz的单位
    /// base_ccy: 交易货币 quote_ccy：计价货币
    #[serde(rename = "tgtCcy")]
    pub tgt_ccy: String,
    /// 最新成交价格
    #[serde(rename = "fillPx")]
    pub fill_px: String,
    /// 最新成交ID
    #[serde(rename = "tradeId", serialize_with="to_str",deserialize_with="from_str")]
    pub trade_id: i64,
    /// 最新成交数量
    /// 对于币币和杠杆，单位为交易货币，如 BTC-USDT, 单位为 BTC；对于市价单，无论tgtCcy是base_ccy，还是quote_ccy，单位均为交易货币；
    /// 对于交割、永续以及期权，单位为张。
    #[serde(rename = "fillSz")]
    pub fill_sz: String,
    /// 最新成交收益，适用于有成交的平仓订单。其他情况均为0。
    #[serde(rename = "fillPnl")]
    pub fill_pnl: String,
    /// 最新成交时间
    #[serde(rename = "fillTime", serialize_with="to_str",deserialize_with="from_str")]
    pub fill_time: i64,
    /// 最新一笔成交的手续费金额或者返佣金额：
    /// 手续费扣除 为 ‘负数’，如 -0.01 ；
    /// 手续费返佣 为 ‘正数’，如 0.01
    #[serde(rename = "fillFee")]
    pub fill_fee: String,
    /// 最新一笔成交的手续费币种或者返佣币种。
    /// 如果fillFee小于0，为手续费币种；如果fillFee大于等于0，为返佣币种
    #[serde(rename = "fillFeeCcy")]
    pub fill_fee_ccy: String,
    /// 最新一笔成交的流动性方向 T：taker M：maker
    #[serde(rename = "execType")]
    pub exec_type: ExecuteType,
    /// 累计成交数量
    /// 对于币币和杠杆，单位为交易货币，如 BTC-USDT, 单位为 BTC；对于市价单，无论tgtCcy是base_ccy，还是quote_ccy，单位均为交易货币；
    /// 对于交割、永续以及期权，单位为张。
    #[serde(rename = "accFillSz")]
    pub acc_fill_sz: String,
    /// 委托单已成交的美元价值
    #[serde(rename = "fillNotionalUsd")]
    pub fill_notional_usd: String,
    /// 成交均价，如果成交数量为0，该字段也为0
    #[serde(rename = "avgPx")]
    pub avg_px: String,
    /// 订单状态
    pub state: OrderState,
    /// 杠杆倍数，0.01到125之间的数值，仅适用于 币币杠杆/交割/永续
    pub lever: String,
    /// 下单附带止盈止损时，客户自定义的策略订单ID
    #[serde(rename = "attachAlgoClOrdId")]
    pub attach_algo_cl_ord_id: String,
    /// 止盈触发价
    #[serde(rename = "tpTriggerPx")]
    pub tp_trigger_px: String,
    /// 止盈触发价类型
    #[serde(rename = "tpTriggerPxType")]
    pub tp_trigger_px_type: TpTriggerPxType,
    /// 止盈委托价，止盈委托价格为-1时，执行市价止盈
    #[serde(rename = "tpOrdPx")]
    pub tp_ord_px: String,
    /// 止损触发价
    #[serde(rename = "slTriggerPx")]
    pub sl_trigger_px: String,
    /// 止损触发价类型
    #[serde(rename = "slTriggerPxType")]
    pub sl_trigger_px_type: TpTriggerPxType,
    /// 止损委托价，止损委托价格为-1时，执行市价止损
    #[serde(rename = "slOrdPx")]
    pub sl_ord_px: String,
    /// 自成交保护ID
    /// 如果自成交保护不适用则返回""
    #[serde(rename = "stpId")]
    pub stp_id: String,
    /// 自成交保护模式
    /// 如果自成交保护不适用则返回""
    #[serde(rename = "stpMode")]
    pub stp_mode: StopMode,
    /// 交易手续费币种
    /// 币币/币币杠杆：如果是买的话，收取的就是BTC；如果是卖的话，收取的就是USDT
    /// 交割/永续/期权 收取的就是保证金
    #[serde(rename = "feeCcy")]
    pub fee_ccy: String,
    /// 订单交易累计的手续费与返佣
    /// 对于币币和杠杆，为订单交易累计的手续费，平台向用户收取的交易手续费，为负数。如： -0.01
    /// 对于交割、永续和期权，为订单交易累计的手续费和返佣
    pub fee: String,
    /// 返佣金币种 ，如果没有返佣金，该字段为“”
    #[serde(rename = "rebateCcy")]
    pub rebate_ccy: String,
    /// 返佣累计金额，仅适用于币币和杠杆，平台向达到指定lv交易等级的用户支付的挂单奖励（返佣），如果没有返佣金，该字段为“”
    pub rebate: String,
    /// 收益，适用于有成交的平仓订单，其他情况均为0
    pub pnl: String,
    /// 订单来源
    /// 13:策略委托单触发后的生成的限价单
    pub source: String,
    /// 订单取消的来源
    /// 有效值及对应的含义是：
    /// 0: 已撤单：系统撤单
    /// 1: 用户主动撤单
    /// 2: 已撤单：预减仓撤单，用户保证金不足导致挂单被撤回
    /// 3: 已撤单：风控撤单，用户保证金不足有爆仓风险，导致挂单被撤回
    /// 4: 已撤单：币种借币量达到平台硬顶，系统已撤回该订单
    /// 6: 已撤单：触发 ADL 撤单，用户保证金率较低且有爆仓风险，导致挂单被撤回
    /// 9: 已撤单：扣除资金费用后可用余额不足，系统已撤回该订单
    /// 13: 已撤单：FOK 委托订单未完全成交，导致挂单被完全撤回
    /// 14: 已撤单：IOC 委托订单未完全成交，仅部分成交，导致部分挂单被撤回
    /// 17: 已撤单：平仓单被撤单，由于仓位已被市价全平
    /// 20: 系统倒计时撤单
    /// 21: 已撤单：相关仓位被完全平仓，系统已撤销该止盈止损订单
    /// 22, 23: 已撤单：只减仓订单仅允许减少仓位数量，系统已撤销该订单
    /// 27: 成交滑点超过5%，触发成交差价保护导致系统撤单
    /// 31: 当前只挂单订单 (Post only) 将会吃掉挂单深度
    /// 32: 自成交保护
    #[serde(rename = "cancelSource")]
    pub cancel_source: String,
    /// 订单修改的来源
    /// 1: 用户主动改单，改单成功
    /// 2: 用户主动改单，并且当前这笔订单被只减仓修改，改单成功
    /// 3: 用户主动下单，并且当前这笔订单被只减仓修改，改单成功
    /// 4: 用户当前已存在的挂单（非当前操作的订单），被只减仓修改，改单成功
    #[serde(rename = "amendSource")]
    pub amend_source: String,
    /// 订单种类分类
    /// normal：普通委托订单种类
    /// twap：TWAP订单种类
    /// adl：ADL订单种类
    /// full_liquidation：爆仓订单种类
    /// partial_liquidation：减仓订单种类
    /// delivery：交割
    /// ddh：对冲减仓类型订单
    pub category: String,
    /// 订单创建时间，Unix时间戳的毫秒数格式，如 1597026383085
    #[serde(rename = "cTime", serialize_with="to_str",deserialize_with="from_str")]
    pub c_time: i64,
    /// 订单更新时间，Unix时间戳的毫秒数格式，如 1597026383085
    #[serde(rename = "uTime", serialize_with="to_str",deserialize_with="from_str")]
    pub u_time: i64,
    /// 修改订单时使用的request ID，如果没有修改，该字段为""
    #[serde(rename = "reqId")]
    pub req_id: String,
    /// 修改订单的结果
    /// -1： 失败
    /// 0：成功
    /// 1：自动撤单（因为修改失败导致订单自动撤销）
    /// 通过API修改订单时，如果cxlOnFail设置为false且修改失败后，则amendResult返回 -1
    /// 通过API修改订单时，如果cxlOnFail设置为true且修改失败后，则amendResult返回1
    /// 通过Web/APP修改订单时，如果修改失败后，则amendResult返回-1
    #[serde(rename = "amendResult")]
    pub amend_result: String,
    /// 是否只减仓，true 或 false
    #[serde(rename = "reduceOnly", serialize_with="to_str",deserialize_with="from_str")]
    pub reduce_only: bool,
    /// 一键借币类型，仅适用于杠杆逐仓的一键借币模式
    /// manual：手动，auto_borrow： 自动借币，auto_repay： 自动还币
    #[serde(rename = "quickMgnType")]
    pub quick_mgn_type: String,
    /// 客户自定义策略订单ID。策略订单触发，且策略单有algoClOrdId时有值，否则为"",
    #[serde(rename = "algoClOrdId")]
    pub algo_cl_ord_id: String,
    /// 策略委托单ID，策略订单触发时有值，否则为""
    #[serde(rename = "algoId")]
    pub algo_id: String,
    #[serde(serialize_with="to_str",deserialize_with="from_str")]
    pub code: i32,
    pub msg: String,
}

#[async_trait]
#[allow(unused)]
pub trait AccountHandler: Send + Sync {
    fn id(&self) -> String;
    async fn on_connected(&self){}
    async fn on_disconnected(&self){}
    async fn on_finish_auth(&self){}
    async fn account_event(&self, events: &Vec<AccountEvent>){}
    async fn order_event(&self, events: &Vec<OrderEvent>){}
    async fn handle_response(&self, resp: &EventResponse){}
}