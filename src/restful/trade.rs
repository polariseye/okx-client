use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::api_enum::APiEnum;
use crate::apikey::OkxAccountClient;
use super::models::*;
use crate::okx_error::*;

#[derive(Serialize, Debug, Deserialize)]
pub struct OrdersPendingFilter {
    #[serde(rename="instType")]
    pub inst_type: Option<InstType>,
    pub uly: Option<String>,
    #[serde(rename="instFamily")]
    pub inst_family: Option<String>,
    #[serde(rename="instId")]
    pub inst_id: Option<String>,
    #[serde(rename="ordType")]
    pub ord_type: Option<Vec<OrderType>>,
    pub state: Option<OrderState>,
    pub after: Option<String>,
    pub before: Option<String>,
    pub limit: Option<String>,
}
impl OkxAccountClient {
    // 获取未成交订单列表
    // 获取当前账户下所有未成交订单信息

    pub async fn trade_orders_pending(
        &self,
        filter: OrdersPendingFilter,
    ) -> Result<Vec<TradeOrdersPending>>
    {
        //  /api/v5/trade/orders-pending
        self.limit_mgr().check_limit(APiEnum::TradeOrdersPending as u32, 1, 60, 2)?;
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        if let Some(inst_type) = &filter.inst_type {
            params.insert("instType".into(), inst_type.clone().into());
        }

        if let Some(uly) = &filter.uly {
            params.insert("uly".into(), uly.into());
        }

        if let Some(inst_family) = &filter.inst_family {
            params.insert("instFamily".into(), inst_family.into());
        }

        if let Some(inst_id) = &filter.inst_id {
            params.insert("instId".into(), inst_id.into());
        }
        if let Some(ord_type) = &filter.ord_type {
           let val:Vec<String> =  ord_type.iter().map(|item|-> String {item.clone().into()}).collect();
            params.insert("ordType".into(), val.join(","));
        }
        if let Some(state) = &filter.state {
            params.insert("state".into(), state.clone().into());
        }
        if let Some(after) = &filter.after {
            params.insert("after".into(), after.into());
        }
        if let Some(before) = &filter.before {
            params.insert("before".into(), before.into());
        }
        if let Some(limit) = &filter.limit {
            params.insert("limit".into(), limit.into());
        }

        self
            .get::<RestApi<TradeOrdersPending>>("/api/v5/trade/orders-pending", &params)
            .await?.to_result()
    }

    //  获取历史订单记录（近七天）
    // 获取最近7天的已经完结状态的订单数据，已经撤销的未成交单 只保留2小时
    // GET /api/v5/trade/orders-history
    pub async fn trade_orders_history(
        &self,
        inst_type: InstType,
        uly: Option<impl Into<String>>,
        inst_family: Option<impl Into<String>>,
        inst_id: Option<impl Into<String>>,
        ord_type: Option<impl Into<String>>,
        state: Option<impl Into<String>>,
        category: Option<impl Into<String>>,
        after: Option<impl Into<String>>,
        before: Option<impl Into<String>>,
        begin: Option<impl Into<String>>,
        end: Option<impl Into<String>>,
        limit: Option<impl Into<String>>,
    ) -> Result<Vec<TradeOrdersHistory>>
    {
        //  /api/index/v3/BTC-USD/constituents
        self.limit_mgr().check_limit(APiEnum::TradeOrdersHistory as u32, 1, 40, 2)?;
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        params.insert("instType".into(), inst_type.into());

        if let Some(uly) = uly {
            params.insert("uly".into(), uly.into());
        }

        if let Some(inst_family) = inst_family {
            params.insert("instFamily".into(), inst_family.into());
        }

        if let Some(inst_id) = inst_id {
            params.insert("instId".into(), inst_id.into());
        }
        if let Some(ord_type) = ord_type {
            params.insert("ordType".into(), ord_type.into());
        }
        if let Some(state) = state {
            params.insert("state".into(), state.into());
        }

        if let Some(category) = category {
            params.insert("category".into(), category.into());
        }

        if let Some(after) = after {
            params.insert("after".into(), after.into());
        }
        if let Some(before) = before {
            params.insert("before".into(), before.into());
        }

        if let Some(begin) = begin {
            params.insert("begin".into(), begin.into());
        }

        if let Some(end) = end {
            params.insert("end".into(), end.into());
        }

        if let Some(limit) = limit {
            params.insert("limit".into(), limit.into());
        }

        self
            .get::<RestApi<TradeOrdersHistory>>("/api/v5/trade/orders-history", &params)
            .await?.to_result()
    }

    // 取消未成交订单

    pub async fn trade_cancel_batch_orders(
        &self,
        inst_id: impl Into<String>,
        order_ids: &[String],
        cl_ord_id: &[String],
    ) -> Result<Vec<TradeCancelBatchOrders>>
    {
        let inst_id = inst_id.into();
        self.limit_mgr().check_limit_with_inst_id(APiEnum::TradeCancelBatchOrders as u32,  &inst_id, (order_ids.len() + cl_ord_id.len()) as u32, 300, 2)?;
        let mut params_vec = Vec::new();
        for item in order_ids {
            let mut params: BTreeMap<String, String> = BTreeMap::new();

            params.insert("instId".into(), inst_id.clone());
            params.insert("ordId".into(), item.clone());
            params_vec.push(params);
        }
        for item in cl_ord_id {
            let mut params: BTreeMap<String, String> = BTreeMap::new();

            params.insert("instId".into(), inst_id.clone());
            params.insert("clOrdId".into(), item.clone());
            params_vec.push(params);
        }

        self
            .post_vec::<RestApi<TradeCancelBatchOrders>>(
                "/api/v5/trade/cancel-batch-orders",
                &params_vec,
            )
            .await?.to_result()
    }

    /// 下单接口
    /// 限速：60次/2s
    /// 跟单交易带单合约的限速：1次/2s
    /// 限速规则（期权以外）：UserID + Instrument ID
    /// 限速规则（只限期权）：UserID + Instrument Family
    pub async fn trade_order(&self, order_obj: OrderRequestInfo) -> Result<TradeOrder>
    {
        self.limit_mgr().check_limit_with_inst_id(APiEnum::TradePlaceOrder as u32,  &order_obj.inst_id, 1, 60, 2)?;

        self
            .post::<RestApi<TradeOrder>>("/api/v5/trade/order", &order_obj)
            .await?.to_result_one()
    }

    /// 下单接口
    /// 限速：300个/2s
    /// 跟单交易带单合约的限速：1个/2s
    /// 限速规则（期权以外）：UserID + Instrument ID
    /// 限速规则（只限期权）：UserID + Instrument Family
    /// 与其他限速按接口调用次数不同，该接口限速按订单的总个数限速。如果单次批量请求中只有一个元素，则算在单个`下单`限速中。
    pub async fn trade_batch_order(&self, inst_id: &str, order_obj: Vec<OrderRequestInfo>) -> Result<Vec<TradeOrder>>
    {
        if order_obj.len() > 20 {
            return Err(OkxError::OutOfMaxOrderSize);
        }
        for item in &order_obj {
            if item.inst_id != item.inst_id {
                return Err(OkxError::MustHaveSameInstId);
            }
        }
        if order_obj.len() == 1 {
            self.limit_mgr().check_limit_with_inst_id(APiEnum::TradePlaceOrder as u32,  &inst_id, 1, 60, 2)?;
        } else {
            self.limit_mgr().check_limit_with_inst_id(APiEnum::TradePlaceBatchOrders as u32,  &inst_id, order_obj.len() as u32, 300, 2)?;
        }

        self
            .post::<RestApi<TradeOrder>>("/api/v5/trade/batch-orders", &order_obj)
            .await?.to_result()
    }

    ///  获取订单信息
    /// 查订单信息
    /// GET /api/v5/trade/order
    /// 限速：60次/2s
    /// 限速规则（期权以外）：UserID + Instrument ID
    /// 限速规则（只限期权）：UserID + Instrument Family
    pub async fn get_trade_order<T>(
        &self,
        inst_id: T,
        ord_id: Option<T>,
        cl_ord_id: Option<T>,
    ) -> Result<Option<TradeOrderGet>>
    where
        T: Into<String>,
    {
        let inst_id = inst_id.into();
        self.limit_mgr().check_limit_with_inst_id(APiEnum::TradeGetOrder as u32,  &inst_id, 1, 60, 2)?;
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        params.insert("instId".into(), inst_id);

        if let Some(ord_id) = ord_id {
            params.insert("ordId".into(), ord_id.into());
        }

        if let Some(cl_ord_id) = cl_ord_id {
            params.insert("clOrdId".into(), cl_ord_id.into());
        }

        self
            .get::<RestApi<TradeOrderGet>>("/api/v5/trade/order", &params)
            .await?.to_result_one_opt()
    }

    ///     修改订单
    /// 修改当前未成交的挂单
    /// 限速：60次/2s
    /// 限速规则：UserID + Instrument ID
    /// POST /api/v5/trade/amend-order

    pub async fn trade_amend_order<T>(
        &self,
        inst_id: T,

        cxl_on_fail: Option<T>,
        ord_id: Option<T>,
        cl_ord_id: Option<T>,

        req_id: Option<T>,
        new_sz: Option<T>,
        new_px: Option<T>,
    ) -> Result<TradeAmendOrder>
    where
        T: Into<String>,
    {
        // let mut params_vec = Vec::new();
        let inst_id = inst_id.into();
        self.limit_mgr().check_limit_with_inst_id(APiEnum::TradeAmendOrder as u32,  &inst_id, 1, 60, 2)?;
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        params.insert("instId".into(), inst_id);

        if let Some(cxl_on_fail) = cxl_on_fail {
            params.insert("cxlOnFail".into(), cxl_on_fail.into());
        }

        if let Some(ord_id) = ord_id {
            params.insert("ordId".into(), ord_id.into());
        }

        if let Some(cl_ord_id) = cl_ord_id {
            params.insert("clOrdId".into(), cl_ord_id.into());
        }

        if let Some(req_id) = req_id {
            params.insert("reqId".into(), req_id.into());
        }

        if let Some(new_sz) = new_sz {
            params.insert("newSz".into(), new_sz.into());
        }

        if let Some(new_px) = new_px {
            params.insert("newPx".into(), new_px.into());
        }

        self
            .post::<RestApi<TradeAmendOrder>>("/api/v5/trade/amend-order", &params)
            .await?.to_result_one()
    }
}
