// use serde::de;
use serde::{Deserialize, Serialize};
use crate::models::de_float_from_str;
use crate::OkxError;
use crate::utils::{from_str, to_str, to_opt_str, from_opt_str};

///////////////////
/// // rest 通用模板

#[derive(Deserialize, Serialize, Debug)]
pub struct RestApi<T> {
    #[serde(serialize_with="to_str",deserialize_with="from_str")]
    pub code: i32,
    pub msg: String,
    #[serde(default="Default::default")]
    pub data: Vec<T>,
}

impl <T> RestApi<T> {
    pub fn is_success(&self) -> bool {
        self.code == 0
    }

    pub fn to_result(self) -> Result<Vec<T>, OkxError> {
        if self.is_success() {
            Ok(self.data)
        } else {
            Err(OkxError::RemoteError { code: self.code, message: self.msg})
        }
    }

    pub fn to_result_one(self) -> Result<T, OkxError> where T: Clone {
        if self.is_success() {
            Ok(self.data[0].clone())
        } else {
            Err(OkxError::RemoteError { code: self.code, message: self.msg})
        }
    }

    pub fn to_result_one_opt(self) -> Result<Option<T>, OkxError> where T: Clone {
        if self.is_success() {
            if self.data.is_empty() {
                Ok(None)
            } else {
                Ok(Some(self.data[0].clone()))
            }
        } else {
            Err(OkxError::RemoteError { code: self.code, message: self.msg})
        }
    }
}

// 查看持仓信息
// GET /api/v5/account/positions

#[derive(Clone,Debug, Deserialize, Serialize)]
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
#[derive(Clone, Debug, Deserialize, Serialize)]
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSetLeverage {
    pub lever: String,
    pub mgn_mode: String,
    pub inst_id: String,
    pub pos_side: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Instrument {
    /// 产品类型
    #[serde(rename = "instType")]
    pub inst_type: InstType,
    /// 产品id， 如 BTC-USD-SWAP
    #[serde(rename = "instId")]
    pub inst_id: String,
    /// 标的指数，如 BTC-USD，仅适用于交割/永续/期权
    pub uly: String,
    /// 交易品种，如 BTC-USD，仅适用于交割/永续/期权
    #[serde(rename = "instFamily")]
    pub inst_family: String,
    /// 交易货币币种，如 BTC-USDT 中的 BTC ，仅适用于币币/币币杠杆
    #[serde(rename = "baseCcy")]
    pub base_ccy: String,
    /// 计价货币币种，如 BTC-USDT 中的USDT ，仅适用于币币/币币杠杆
    #[serde(rename = "quoteCcy")]
    pub quote_ccy: String,
    /// 盈亏结算和保证金币种，如 BTC 仅适用于交割/永续/期权
    #[serde(rename = "settleCcy")]
    pub settle_ccy: String,
    /// 合约面值，仅适用于交割/永续/期权
    #[serde(rename = "ctVal")]
    pub ct_val: String,
    /// 合约乘数，仅适用于交割/永续/期权
    #[serde(rename = "ctMult")]
    pub ct_mult: String,
    /// 合约面值计价币种，仅适用于交割/永续/期权
    #[serde(rename = "ctValCcy")]
    pub ct_val_ccy: String,
    /// 期权类型，C或P 仅适用于期权
    #[serde(rename = "optType")]
    pub opt_type: String,
    /// 行权价格，仅适用于期权
    pub stk: String,
    /// 上线日期
    /// Unix时间戳的毫秒数格式，如 1597026383085
    #[serde(rename = "listTime", serialize_with="to_opt_str",deserialize_with="from_opt_str")]
    pub list_time: Option<i64>,
    /// 交割/行权日期，仅适用于交割 和 期权
    /// Unix时间戳的毫秒数格式，如 1597026383085
    #[serde(rename = "expTime", serialize_with="to_opt_str",deserialize_with="from_opt_str")]
    pub exp_time: Option<i64>,
    /// 该instId支持的最大杠杆倍数，不适用于币币、期权
    pub lever: String,
    /// 下单价格精度，如 0.0001
    /// 对于期权来说，是梯度中的最小下单价格精度，如果想要获取期权价格梯度，请使用"获取期权价格梯度"接口
    #[serde(rename = "tickSz")]
    pub tick_sz: String,
    /// 下单数量精度，如 BTC-USDT-SWAP：1
    #[serde(rename = "lotSz")]
    pub lot_sz: String,
    /// 最小下单数量,
    /// 合约的数量单位是“张”，现货的数量单位是“交易货币”
    #[serde(rename = "minSz")]
    pub min_sz: String,
    /// linear：正向合约
    /// inverse：反向合约
    /// 仅适用于交割/永续
    #[serde(rename = "ctType")]
    pub ct_type: String,
    /// 合约日期别名
    /// this_week：本周
    /// next_week：次周
    /// quarter：季度
    /// next_quarter：次季度
    /// 仅适用于交割
    pub alias: String,
    /// 产品状态
    /// live：交易中
    /// suspend：暂停中
    /// preopen：预上线，如：交割和期权的新合约在 live 之前，会有 preopen 状态
    /// test：测试中（测试产品，不可交易）
    pub state: InstrumentState,
    /// 合约或现货限价单的单笔最大委托数量,
    /// 合约的数量单位是“张”，现货的数量单位是“交易货币”
    #[serde(rename = "maxLmtSz")]
    pub max_lmt_sz: String,
    /// 合约或现货市价单的单笔最大委托数量,
    /// 合约的数量单位是“张”，现货的数量单位是“USDT”
    #[serde(rename = "maxMktSz")]
    pub max_mkt_sz: String,
    /// 合约或现货时间加权单的单笔最大委托数量,
    /// 合约的数量单位是“张”，现货的数量单位是“交易货币”
    #[serde(rename = "maxTwapSz")]
    pub max_twap_sz: String,
    /// 合约或现货冰山委托的单笔最大委托数量,
    /// 合约的数量单位是“张”，现货的数量单位是“交易货币”
    #[serde(rename = "maxIcebergSz")]
    pub max_iceberg_sz: String,
    /// 合约或现货计划委托委托的单笔最大委托数量,
    /// 合约的数量单位是“张”，现货的数量单位是“交易货币”
    #[serde(rename = "maxTriggerSz")]
    pub max_trigger_sz: String,
    /// 合约或现货止盈止损市价委托的单笔最大委托数量,
    /// 合约的数量单位是“张”，现货的数量单位是“USDT”
    #[serde(rename = "maxStopSz")]
    pub max_stop_sz: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum InstrumentState {
    /// 交易中
    #[serde(rename="live")]
    Live,
    /// 暂停中
    #[serde(rename="suspend")]
    Suspend,
    /// 预上线，如：交割和期权的新合约在 live 之前，会有 preopen 状态
    #[serde(rename="preopen")]
    Preopen,
    /// 测试中（测试产品，不可交易）
    #[serde(rename="test")]
    Test,
}

// 获取所有产品行情信息
// 获取产品行情信息
#[derive(Clone, Debug, Deserialize, Serialize)]
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
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    #[serde(serialize_with="to_str",deserialize_with="from_str")]
    pub ts: i64,
}

///
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketBooks {
    pub asks: Vec<MarketBooksItemData>,
    pub bids: Vec<MarketBooksItemData>,
    pub ts: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrdersPending {
    pub inst_type: InstType,
    pub inst_id: String,
    pub tgt_ccy: String,
    pub ccy: String,
    pub ord_id: String,
    pub cl_ord_id: String,
    pub tag: String,
    pub px: String,
    pub sz: String,
    pub pnl: String,
    pub ord_type: OrderType,
    pub side: TradeSide,
    pub pos_side: PositionSide,
    pub td_mode: TradeMode,
    pub acc_fill_sz: String,

    pub fill_px: String,
    #[serde(rename = "tradeId", serialize_with="to_str",deserialize_with="from_str")]
    pub trade_id: i64,
    pub fill_sz: String,
    #[serde(rename = "fillTime", serialize_with="to_str",deserialize_with="from_str")]
    pub fill_time: i64,
    pub avg_px: String,
    pub state: OrderState,

    pub lever: String,
    pub tp_trigger_px: String,
    pub tp_trigger_px_type: TpTriggerPxType,
    pub sl_trigger_px: String,
    pub sl_trigger_px_type: TpTriggerPxType,

    pub sl_ord_px: String,
    pub tp_ord_px: String,
    pub fee_ccy: String,
    pub fee: String,
    pub rebate_ccy: String,
    pub source: String,
    pub rebate: String,
    pub category: String,

    pub stp_id: String,
    pub stp_mode: StopMode,

    #[serde(rename = "reduceOnly", serialize_with="to_str",deserialize_with="from_str")]
    pub reduce_only: bool,
    pub quick_mgn_type: String,
    #[serde(serialize_with="to_str",deserialize_with="from_str")]
    pub u_time: i64,
    #[serde(serialize_with="to_str",deserialize_with="from_str")]
    pub c_time: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrdersHistory {
    pub inst_type: String,
    pub inst_id: String,

    pub ord_type: OrderType,
    //     订单状态
    // canceled：撤单成功
    // filled：完全成交
    pub state: OrderState,
    #[serde(deserialize_with = "de_float_from_str")]
    pub pnl: f32,
}

// 批量撤单
// 撤销未完成的订单，每次最多可以撤销20个订单。请求参数应该按数组格式传递。
// POST /api/v5/trade/cancel-batch-orders
#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeAmendOrder {
    pub ord_id: String,    //持仓方向
    pub cl_ord_id: String, //持仓方向
    pub req_id: String,    //持仓方向

    pub s_code: String, //持仓方向
    pub s_msg: String,  //持仓方向
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum TradeMode {
    /// isolated：逐仓
    #[serde(rename="isolated")]
    Isolated,
    /// 全仓
    #[serde(rename="cross")]
    Cross,
    /// 非保证金模式：cash：非保证金
    #[serde(rename="cash")]
    Cash
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum TradeSide {
    #[serde(rename="buy")]
    Buy,
    #[serde(rename="sell")]
    Sell,
}

/// 持仓方向 在开平仓模式下必填，且仅可选择 long 或 short。 仅适用交割、永续。
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum PositionSide {
    #[serde(rename="")]
    None,
    #[serde(rename="long")]
    Long,
    #[serde(rename="short")]
    Short,
    #[serde(rename="net")]
    Net
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum OrderType {
    /// 市价单
    #[serde(rename="market")]
    Market,
    /// 限价单
    #[serde(rename="limit")]
    Limit,
    /// 只做maker单
    #[serde(rename="post_only")]
    PostOnly,
    /// 全部成交或立即取消
    #[serde(rename="fok")]
    Fok,
    /// 立即成交并取消剩余
    #[serde(rename="ioc")]
    Ioc,
    /// 市价委托立即成交并取消剩余（仅适用交割、永续）
    #[serde(rename="optimal_limit_ioc")]
    OptimalLimitIoc,
    /// 做市商保护(仅适用于组合保证金账户模式下的期权订单)
    #[serde(rename="mmp")]
    Mmp,
    /// 做市商保护且只做maker单(仅适用于组合保证金账户模式下的期权订单)
    #[serde(rename="mmp_and_post_only")]
    MmpAndPostOnly,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum StopMode {
    #[serde(rename="")]
    None,
    #[serde(rename="cancel_maker")]
    CancelMaker,
    #[serde(rename="cancel_taker")]
    CancelTaker,
    #[serde(rename="cancel_both")]
    CancelBoth
}

/// 止盈触发价类型
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum TpTriggerPxType {
    #[serde(rename="")]
    None,
    // 最新价格
    #[serde(rename="last")]
    Last,
    // 指数价格
    #[serde(rename="index")]
    Index,
    // 标记价格
    #[serde(rename="mark")]
    Mark,
}

/// 一键借币类型
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum QuickMgnType{
    #[serde(rename="")]
    None,
    /// 手动
    #[serde(rename="manual")]
    Manual,
    ///  自动借币
    #[serde(rename="auto_borrow")]
    AutoBorrow,
    ///  自动还币
    #[serde(rename="auto_repay")]
    AutoRepay,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum OrderState {
    #[serde(rename="live")]
    Live,
    #[serde(rename="partially_filled")]
    PartiallyFilled,
    #[serde(rename="filled")]
    Filled,
    /// 做市商保护机制导致的自动撤单
    #[serde(rename="mmp_canceled")]
    MmpCanceled,
    /// 撤单成功
    #[serde(rename="canceled")]
    Canceled,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OrderRequestInfo {
    /// 产品ID，如 BTC-USDT
    #[serde(rename = "instId")]
    pub inst_id: String,
    /// 交易模式
    /// 保证金模式：isolated：逐仓 ；cross：全仓
    /// 非保证金模式：cash：非保证金
    #[serde(rename = "tdMode")]
    pub td_mode: TradeMode,
    /// 保证金币种，仅适用于单币种保证金模式下的全仓杠杆订单
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ccy: Option<String>,
    /// 客户自定义订单ID
    /// 字母（区分大小写）与数字的组合，可以是纯字母、纯数字且长度要在1-32位之间。
    #[serde(rename = "clOrdId", skip_serializing_if = "Option::is_none")]
    pub cl_ord_id: Option<String>,
    /// 订单标签
    /// 字母（区分大小写）与数字的组合，可以是纯字母、纯数字且长度要在1-16位之间。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// 订单方向 buy：买， sell：卖
    pub side: TradeSide,
    /// 持仓方向
    /// 在开平仓模式下必填，且仅可选择 long 或 short。 仅适用交割、永续。
    #[serde(rename="posSide", skip_serializing_if = "Option::is_none")]
    pub pos_side: Option<PositionSide>,
    /// 订单类型
    /// market：市价单
    /// limit：限价单
    /// post_only：只做maker单
    /// fok：全部成交或立即取消
    /// ioc：立即成交并取消剩余
    /// optimal_limit_ioc：市价委托立即成交并取消剩余（仅适用交割、永续）
    /// mmp：做市商保护(仅适用于组合保证金账户模式下的期权订单)
    /// mmp_and_post_only：做市商保护且只做maker单(仅适用于组合保证金账户模式下的期权订单)
    #[serde(rename="ordType")]
    pub order_type: OrderType,
    /// 委托数量
    pub sz: String,
    /// 委托价格，仅适用于limit、post_only、fok、ioc、mmp类型的订单
    #[serde(skip_serializing_if = "Option::is_none")]
    pub px: Option<String>,
    /// 是否只减仓，true 或 false，默认false
    /// 仅适用于币币杠杆，以及买卖模式下的交割/永续
    /// 仅适用于单币种保证金模式和跨币种保证金模式
    #[serde(rename = "reduceOnly", skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    /// 市价单委托数量sz的单位，仅适用于币币市价订单
    /// base_ccy: 交易货币 ；quote_ccy：计价货币
    /// 买单默认quote_ccy， 卖单默认base_ccy
    #[serde(rename = "tgtCcy", skip_serializing_if = "Option::is_none")]
    pub tgt_ccy: Option<String>,
    /// 是否禁止币币市价改单，true 或 false，默认false
    /// 为true时，余额不足时，系统不会改单，下单会失败，仅适用于币币市价单
    #[serde(rename="banAmend", skip_serializing_if = "Option::is_none")]
    pub ban_amend: Option<bool>,
    /// 下单附带止盈止损时，客户自定义的策略订单ID
    /// 字母（区分大小写）与数字的组合，可以是纯字母、纯数字且长度要在1-32位之间。
    /// 订单完全成交，下止盈止损委托单时，该值会传给algoClOrdId
    #[serde(rename="attachAlgoClOrdId", skip_serializing_if = "Option::is_none")]
    pub attach_algo_cl_ord_id: Option<String>,
    /// 止盈触发价，如果填写此参数，必须填写 止盈委托价
    #[serde(rename="tpTriggerPx", skip_serializing_if = "Option::is_none")]
    pub tp_trigger_px: Option<String>,
    /// 止盈委托价，如果填写此参数，必须填写 止盈触发价
    /// 委托价格为-1时，执行市价止盈
    #[serde(rename="tpOrdPx", skip_serializing_if = "Option::is_none")]
    pub tp_ord_px: Option<String>,
    #[serde(rename="slTriggerPx", skip_serializing_if = "Option::is_none")]
    pub sl_trigger_px: Option<String>,
    /// 止损委托价，如果填写此参数，必须填写 止损触发价
    /// 委托价格为-1时，执行市价止损
    #[serde(rename="slOrdPx", skip_serializing_if = "Option::is_none")]
    pub sl_ord_px: Option<String>,
    /// 自成交保护ID。来自同一个母账户配着同一个ID的订单不能自成交
    /// 用户自定义1<=x<=999999999的整数
    #[serde(rename="stpId", skip_serializing_if = "Option::is_none")]
    pub stp_id: Option<String>,
    /// 自成交保护模式
    /// 预设 cancel maker
    /// cancel_maker,cancel_taker, cancel_both
    /// Cancel both不支持FOK
    #[serde(rename="stpMode", skip_serializing_if = "Option::is_none")]
    pub stp_mode: Option<StopMode>,
    /// 止盈触发价类型
    /// last：最新价格
    /// index：指数价格
    /// mark：标记价格
    /// 默认为last
    #[serde(rename="tpTriggerPxType", skip_serializing_if = "Option::is_none")]
    pub tp_trigger_px_type: Option<TpTriggerPxType>,
    /// 止损触发价类型
    /// last：最新价格
    /// index：指数价格
    /// mark：标记价格
    /// 默认为last
    #[serde(rename="tpTriggerPxType", skip_serializing_if = "Option::is_none")]
    pub sl_trigger_px_type: Option<TpTriggerPxType>,
    /// 一键借币类型，仅适用于杠杆逐仓的一键借币模式：
    /// manual：手动，auto_borrow： 自动借币，auto_repay： 自动还币
    /// 默认是manual：手动
    #[serde(rename="quickMgnType", skip_serializing_if = "Option::is_none")]
    pub quick_mgn_type: Option<QuickMgnType>,
}

impl OrderRequestInfo {
    pub fn new_spot_limit_order(inst_id: &str, side: TradeSide, sz: String, px: String , cl_ord_id: Option<String>, tag: Option<String>) -> Self {
        Self {
            inst_id: inst_id.to_string(),
            td_mode: TradeMode::Cash,
            ccy: None,
            cl_ord_id,
            tag,
            side,
            pos_side: None,
            order_type: OrderType::Limit,
            sz,
            px: Some(px),
            reduce_only: None,
            tgt_ccy: None,
            ban_amend: None,
            attach_algo_cl_ord_id: None,
            tp_trigger_px: None,
            tp_ord_px: None,
            sl_trigger_px: None,
            sl_ord_px: None,
            stp_id: None,
            stp_mode: None,
            tp_trigger_px_type: None,
            sl_trigger_px_type: None,
            quick_mgn_type: None,
        }
    }
}

/// 产品类型
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum InstType {
    /// 币币
    #[serde(rename="SPOT")]
    Spot,
    /// 币币杠杆
    #[serde(rename="MARGIN")]
    Margin,
    /// 永续合约
    #[serde(rename="SWAP")]
    Swap,
    /// 交割合约
    #[serde(rename="FUTURES")]
    Futures,
    /// 期权
    #[serde(rename="OPTION")]
    Option,
    /// 所有
    #[serde(rename="ANY")]
    Any,
}

// 获取订单信息
// 查订单信息

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOrderGet {
    pub state: String, //订单状态  filled
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BalanceDetailItem {
    /// 币种
    pub ccy: String,
    /// 币种总权益
    pub eq: String,
    /// 币种余额
    #[serde(rename = "cashBal")]
    pub cash_bal: String,
    #[serde(rename = "uTime", serialize_with="to_str",deserialize_with="from_str")]
    pub u_time: i64,
    /// 币种逐仓仓位权益
    /// 适用于单币种保证金模式和跨币种保证金模式和组合保证金模式
    #[serde(rename = "isoEq")]
    pub iso_eq: String,
    /// 可用保证金
    /// 适用于单币种保证金模式和跨币种保证金模式和组合保证金模式
    #[serde(rename = "availEq")]
    pub avail_eq: String,
    /// 美金层面币种折算权益
    #[serde(rename = "disEq")]
    pub dis_eq: String,
    /// 可用余额
    /// 适用于简单交易模式、单币种保证金模式、跨币种保证金模式和组合保证金模式
    #[serde(rename = "availBal")]
    pub avail_bal: String,
    /// 币种占用金额
    #[serde(rename = "frozenBal")]
    pub frozen_bal: String,
    /// 挂单冻结数量
    #[serde(rename = "ordFrozen")]
    pub ord_frozen: String,
    /// 币种负债额
    /// 适用于跨币种保证金模式和组合保证金模式
    pub liab: String,
    /// 未实现盈亏
    /// 适用于单币种保证金模式和跨币种保证金模式和组合保证金模式
    pub upl: String,
    /// 由于仓位未实现亏损导致的负债
    /// 适用于跨币种保证金模式和组合保证金模式
    #[serde(rename = "uplLiab")]
    pub upl_liab: String,
    /// 币种全仓负债额
    /// 适用于跨币种保证金模式和组合保证金模式
    #[serde(rename = "crossLiab")]
    pub cross_liab: String,
    /// 币种逐仓负债额
    /// 适用于跨币种保证金模式和组合保证金模式
    #[serde(rename = "isoLiab")]
    pub iso_liab: String,
    /// 保证金率
    /// 适用于单币种保证金模式
    #[serde(rename = "mgnRatio")]
    pub mgn_ratio: String,
    #[serde(rename = "eqUsd")]
    pub eq_usd: String,
    /// 计息，应扣未扣利息。
    /// 适用于跨币种保证金模式和组合保证金模式
    pub interest: String,
    /// 当前负债币种触发系统自动换币的风险
    /// 0、1、2、3、4、5其中之一，数字越大代表您的负债币种触发自动换币概率越高
    /// 适用于跨币种保证金模式和组合保证金模式
    #[serde(serialize_with="to_str",deserialize_with="from_str")]
    pub twap: i32,
    /// 币种最大可借
    /// 适用于跨币种保证金模式和组合保证金模式 的全仓
    #[serde(rename = "maxLoan")]
    pub max_loan: String,
    /// 币种杠杆倍数
    /// 适用于单币种保证金模式
    #[serde(rename = "notionalLever")]
    pub notional_lever: String,
    /// 币种权益美金价值
    #[serde(rename = "stgyEq")]
    pub stgy_eq: String,
    /// 逐仓未实现盈亏
    /// 适用于单币种保证金模式和跨币种保证金模式和组合保证金模式
    #[serde(rename = "isoUpl")]
    pub iso_upl: String,
    /// 现货对冲占用数量
    /// 适用于组合保证金模式
    #[serde(rename = "spotInUseAmt")]
    pub spot_in_use_amt: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountBalance {
    /// 美金层面有效保证金
    /// 适用于跨币种保证金模式和组合保证金模式
    #[serde(rename = "adjEq")]
    pub adj_eq: String,
    pub details: Vec<BalanceDetailItem>,
    /// 美金层面占用保证金
    /// 适用于跨币种保证金模式和组合保证金模式
    pub imr: String,
    /// 美金层面逐仓仓位权益
    /// 适用于单币种保证金模式和跨币种保证金模式和组合保证金模式
    #[serde(rename = "isoEq")]
    pub iso_eq: String,
    /// 美金层面保证金率
    /// 适用于跨币种保证金模式 和组合保证金模式
    #[serde(rename = "mgnRatio")]
    pub mgn_ratio: String,
    /// 美金层面维持保证金
    /// 适用于跨币种保证金模式和组合保证金模式
    pub mmr: String,
    /// 以美金价值为单位的持仓数量，即仓位美金价值
    /// 适用于跨币种保证金模式和组合保证金模式
    #[serde(rename = "notionalUsd")]
    pub notional_usd: String,
    /// 金层面全仓挂单占用保证金
    /// 仅适用于跨币种保证金模式
    #[serde(rename = "ordFroz")]
    pub ord_froz: String,
    /// 美金层面权益
    #[serde(rename = "totalEq")]
    pub total_eq: String,
    /// 账户信息的更新时间，Unix时间戳的毫秒数格式，如 1597026383085
    #[serde(rename = "uTime", serialize_with="to_str",deserialize_with="from_str")]
    pub u_time: i64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum ExecuteType{
    #[serde(rename="")]
    None,
    #[serde(rename="T")]
    Taker,
    #[serde(rename="M")]
    Maker
}

#[derive(Clone,Serialize, Deserialize, Debug)]
pub struct Trade {
    /// 产品ID，如 BTC-USD-180216
    #[serde(rename = "instId")]
    pub inst_id: String,
    /// 成交ID
    #[serde(rename = "tradeId", serialize_with="to_str",deserialize_with="from_str")]
    pub trade_id: i64,
    /// 成交价格
    pub px: String,
    /// 成交数量
    pub sz: String,
    /// 成交方向，buy sell
    pub side: String,
    /// 成交时间，Unix时间戳的毫秒数格式，如 1597026383085
    #[serde(serialize_with="to_str",deserialize_with="from_str")]
    pub ts: i64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub enum GreeksType {
    /// 币本位
    #[serde(rename="PA")]
    Pa,
    /// 美元本位
    #[serde(rename="BS")]
    Bs
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountConfig {
    /// 当前请求的账户ID，账户uid和app上的一致
    pub uid: String,
    /// 当前请求的母账户ID
    /// 如果 uid = mainUid，代表当前账号为母账户；如果 uid != mainUid，代表当前账户为子账户。
    #[serde(rename = "mainUid")]
    pub main_uid: String,
    /// 账户层级
    /// 1：简单交易模式，2：单币种保证金模式 ，3：跨币种保证金模式 ，4：组合保证金模式
    #[serde(rename = "acctLv", serialize_with="to_str",deserialize_with="from_str")]
    pub acct_lv: u32,
    /// 持仓方式
    /// long_short_mode：开平仓模式 net_mode：买卖模式
    /// 仅适用交割/永续
    #[serde(rename = "posMode")]
    pub pos_mode: String,
    /// 是否自动借币
    /// true：自动借币 false：非自动借币
    #[serde(rename = "autoLoan")]
    pub auto_loan: bool,
    /// 当前希腊字母展示方式
    /// PA：币本位 BS：美元本位
    #[serde(rename = "greeksType")]
    pub greeks_type: GreeksType,
    /// 当前在平台上真实交易量的用户等级，例如 lv1
    // #[serde(serialize_with="to_str",deserialize_with="from_str")]
    pub level: Level,
    /// 特约用户的临时体验用户等级，例如 lv3
    #[serde(rename = "levelTmp")]
    pub level_tmp: Level,
    /// 衍生品的逐仓保证金划转模式
    /// automatic：开仓划转 autonomy：自主划转
    #[serde(rename = "ctIsoMode")]
    pub ct_iso_mode: String,
    /// 币币杠杆的逐仓保证金划转模式
    /// automatic：开仓划转 quick_margin：一键借币（对于新的账户，包括新的子账户，有些默认是开仓划转，另外的默认是一键借币）
    #[serde(rename = "mgnIsoMode")]
    pub mgn_iso_mode: String,
    /// 现货对冲类型
    /// 1：现货对冲模式U模式 2：现货对冲模式币模式 3：非现货对冲模式
    /// 适用于组合保证金模式
    #[serde(rename = "spotOffsetType")]
    pub spot_offset_type: String,
    /// 用户角色。
    /// 0：普通用户；1：带单者；2：跟单者
    #[serde(rename = "roleType")]
    pub role_type: String,
    /// 当前账号已经设置的带单合约，仅适用于带单者
    #[serde(rename = "traderInsts")]
    pub trader_insts: Vec<String>,
    /// 是否开通期权交易
    /// 0 未开通，1 已经开通
    #[serde(rename = "opAuth")]
    pub op_auth: String,
    /// 母账户KYC等级
    /// 0: 未认证 1: 已完成 level 1 认证, 2: 已完成 level 2 认证, 3: 已完成 level 3认证.
    /// 如果请求来自子账户, kycLv 为其母账户的等级.
    /// 如果请求来自母账户, kycLv 为当前请求的母账户等级.
    #[serde(rename = "kycLv")]
    pub kyc_lv: String,
    /// 当前请求API key的备注名，不超过50位字母（区分大小写）或数字，可以是纯字母或纯数字。
    pub label: String,
    /// 当前请求API key绑定的ip地址，多个ip用半角逗号隔开，如：117.37.203.58,117.37.203.57。
    /// 如果没有绑定ip，会返回空字符串""
    pub ip: String,
    /// 当前请求的 API key权限 read_only：只读；trade ：交易；withdraw: 提币
    pub perm: String,
}
impl AccountConfig {
    pub fn permission(&self) -> Vec<ApiKeyPermission>{
        let mut result = vec![];
        for item in self.perm.split(",") {
            match item {
                "read_only" => result.push(ApiKeyPermission::ReadOnly),
                "trade" => result.push(ApiKeyPermission::Trade),
                "withdraw" => result.push(ApiKeyPermission::Withdraw),
                _ => {

                }
            }
        }

        result
    }

    pub fn is_main_account(&self) -> bool {
        self.main_uid == self.uid
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Level {
    #[serde(rename="")]
    None,
    #[serde(rename="Lv1")]
    Lv1,
    #[serde(rename="Lv2")]
    Lv2,
    #[serde(rename="Lv3")]
    Lv3,
    #[serde(rename="Lv4")]
    Lv4,
    #[serde(rename="Lv5")]
    Lv5,
    #[serde(rename="Lv6")]
    Lv6,
    #[serde(rename="Lv7")]
    Lv7,
    #[serde(rename="Lv8")]
    Lv8,
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Hash, Copy, Clone)]
pub enum ApiKeyPermission{
    ReadOnly,
    Trade,
    Withdraw
}

macro_rules! impl_to_str {
    ($($arg:tt)*) => {
        $(
        impl Into<String> for $arg {
            fn into(self) -> String {
                let str = serde_json::to_string(&self).unwrap();
                serde_json::from_str(&str).unwrap()
            }
        }
        )*
    };
}

impl_to_str!(
    TradeMode
    TradeSide
    PositionSide
    OrderType
    StopMode
    TpTriggerPxType
    QuickMgnType
    OrderState
    InstType
    InstrumentState
    ExecuteType
);