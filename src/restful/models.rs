// use serde::de;
use serde::{Deserialize, Serialize};

use crate::models::de_float_from_str;

///////////////////
/// // rest 通用模板

#[derive(Deserialize, Serialize, Debug)]
pub struct RestApi<T> {
    pub code: String,
    pub msg: String,
    pub data: Vec<T>,
}

// 查看持仓信息
// GET /api/v5/account/positions

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountPositions {
    pub mgn_mode: String, //保证金模式

    pub pos_side: String, //持仓方向

    // 持仓数量，逐仓自主划转模式下，转入保证金后会产生pos为0的仓位
    #[serde(deserialize_with = "de_float_from_str")]
    pub pos: f32,
    // 可平仓数量，适用于 币币杠杆,交割/永续（开平仓模式），期权（交易账户及保证金账户逐仓）。
    #[serde(deserialize_with = "de_float_from_str")]
    pub avail_pos: f32,
}

// 查看历史持仓信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountPositionsHistory {
    pub inst_type: String, //持仓方向
    pub inst_id: String,   //持仓方向
    pub mgn_mode: String,  //持仓方向
    #[serde(rename = "type")]
    pub ptype: String, //持仓方向

    #[serde(deserialize_with = "de_float_from_str")]
    pub pnl: f32, // 平仓收益额

                  // #[serde(deserialize_with = "de_float_from_str")]
                  // // 持仓数量，逐仓自主划转模式下，转入保证金后会产生pos为0的仓位
                  // pub pos: f32,
}

// 设置杠杆倍数
// POST /api/v5/account/set-leverage
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSetLeverage {
    pub lever: String,
    pub mgn_mode: String,
    pub inst_id: String,
    pub pos_side: String,
}

// 获取所有产品行情信息
// 获取产品行情信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketTickers {
    pub inst_type: String,
    pub inst_id: String,
    pub last: String,
    #[serde(deserialize_with = "de_float_from_str")]
    pub ask_px: f32,
    pub ask_sz: String,
    #[serde(deserialize_with = "de_float_from_str")]
    pub bid_px: f32,
    pub bid_sz: String,
    pub open24h: String,
    pub high24h: String,
    pub low24h: String,
    pub vol_ccy24h: String,
    pub vol24h: String,
    pub sod_utc0: String,
    pub sod_utc8: String,
    pub ts: String,
}

// 获取单个产品行情信息
// 获取产品行情信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketTicker {
    pub inst_type: String,
    pub inst_id: String,
    pub last: String,
    #[serde(deserialize_with = "de_float_from_str")]
    pub ask_px: f32,
    pub ask_sz: String,
    #[serde(deserialize_with = "de_float_from_str")]
    pub bid_px: f32,
    pub bid_sz: String,
    pub open24h: String,
    pub high24h: String,
    pub low24h: String,
    pub vol_ccy24h: String,
    pub vol24h: String,
    pub sod_utc0: String,
    pub sod_utc8: String,
    pub ts: String,
}

///
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketBooks {
    pub asks: Vec<MarketBooksItemData>,
    pub bids: Vec<MarketBooksItemData>,
    pub ts: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketBooksItemData {
    #[serde(deserialize_with = "de_float_from_str")]
    pub price: f32,
    #[serde(deserialize_with = "de_float_from_str")]
    pub sz: f32,
    #[serde(deserialize_with = "de_float_from_str")]
    pub ignore: f32,
    #[serde(deserialize_with = "de_float_from_str")]
    pub count: f32,
}

// 获取未成交订单列表
// 获取当前账户下所有未成交订单信息
// GET /api/v5/trade/orders-pending
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrdersPending {
    pub inst_type: String,
    pub inst_id: String,
    pub tgt_ccy: String,
    pub ccy: String,
    pub ord_id: String,
    pub cl_ord_id: String,
    pub tag: String,
    pub px: String,
    pub sz: String,
    pub pnl: String,
    pub ord_type: String,
    pub side: String,
    pub pos_side: String,
    pub td_mode: String,
    pub acc_fill_sz: String,

    pub fill_px: String,
    pub trade_id: String,
    pub fill_sz: String,
    pub fill_time: String,
    pub avg_px: String,
    pub state: String,

    pub lever: String,
    pub tp_trigger_px: String,
    pub tp_trigger_px_type: String,
    pub sl_trigger_px: String,
    pub sl_trigger_px_type: String,

    pub sl_ord_px: String,
    pub tp_ord_px: String,
    pub fee_ccy: String,
    pub fee: String,
    pub rebate_ccy: String,
    pub source: String,
    pub rebate: String,
    pub category: String,

    pub reduce_only: String,
    pub quick_mgn_type: String,
    pub u_time: String,
    pub c_time: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrdersHistory {
    pub inst_type: String,
    pub inst_id: String,

    pub ord_type: String,
    //     订单状态
    // canceled：撤单成功
    // filled：完全成交
    pub state: String,
    #[serde(deserialize_with = "de_float_from_str")]
    pub pnl: f32,
}

// 批量撤单
// 撤销未完成的订单，每次最多可以撤销20个订单。请求参数应该按数组格式传递。
// POST /api/v5/trade/cancel-batch-orders
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeCancelBatchOrders {
    pub ord_id: String,    //持仓方向
    pub cl_ord_id: String, //持仓方向
    pub s_code: String,    //持仓方向
    pub s_msg: String,     //持仓方向
}

// 下单
// 只有当您的账户有足够的资金才能下单。
// 该接口支持带单合约的下单，但不支持为带单合约平仓

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrder {
    pub ord_id: String,    //持仓方向
    pub cl_ord_id: String, //持仓方向
    pub s_code: String,    //持仓方向
    pub tag: String,
    pub s_msg: String, //持仓方向
}

// 修改订单
// 修改当前未成交的挂单
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeAmendOrder {
    pub ord_id: String,    //持仓方向
    pub cl_ord_id: String, //持仓方向
    pub req_id: String,    //持仓方向

    pub s_code: String, //持仓方向
    pub s_msg: String,  //持仓方向
}

// 获取订单信息
// 查订单信息

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrderGet {
    pub state: String, //订单状态  filled
}
