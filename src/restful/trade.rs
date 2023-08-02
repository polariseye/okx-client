use std::collections::BTreeMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::apikey::OkxAccountClient;
use super::models::*;

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
    ) -> Result<RestApi<TradeOrdersPending>>
    {
        //  /api/index/v3/BTC-USD/constituents
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

        Ok(self
            .get::<RestApi<TradeOrdersPending>>("/api/v5/trade/orders-pending", &params)
            .await?)
    }

    //  获取历史订单记录（近七天）
    // 获取最近7天的已经完结状态的订单数据，已经撤销的未成交单 只保留2小时
    // GET /api/v5/trade/orders-history
    pub async fn trade_orders_history<T>(
        &self,
        inst_type: T,
        uly: Option<T>,
        inst_family: Option<T>,
        inst_id: Option<T>,
        ord_type: Option<T>,
        state: Option<T>,
        category: Option<T>,
        after: Option<T>,
        before: Option<T>,
        begin: Option<T>,
        end: Option<T>,
        limit: Option<T>,
    ) -> Result<RestApi<TradeOrdersHistory>>
    where
        T: Into<String>,
    {
        //  /api/index/v3/BTC-USD/constituents
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

        Ok(self
            .get::<RestApi<TradeOrdersHistory>>("/api/v5/trade/orders-history", &params)
            .await?)
    }

    // 取消未成交订单

    pub async fn trade_cancel_batch_orders<T>(
        &self,
        inst_id: Option<T>,
        order_ids: &Vec<String>,
        cl_ord_id: Option<T>,
    ) -> Result<RestApi<TradeCancelBatchOrders>>
    where
        T: Into<String> + Clone,
    {
        let mut params_vec = Vec::new();
        for item in order_ids {
            let mut params: BTreeMap<String, String> = BTreeMap::new();

            if let Some(inst_id) = &inst_id {
                params.insert("instId".into(), inst_id.clone().into());
            }

            if let Some(cl_ord_id) = &cl_ord_id {
                params.insert("clOrdId".into(), cl_ord_id.clone().into());
            }

            params.insert("ordId".into(), item.clone());
            params_vec.push(params);
        }

        Ok(self
            .post_vec::<RestApi<TradeCancelBatchOrders>>(
                "/api/v5/trade/cancel-batch-orders",
                &params_vec,
            )
            .await?)
    }

    // 下单接口

    pub async fn trade_order(&self, order_obj: OrderRequestInfo) -> Result<RestApi<TradeOrder>>
    {
        Ok(self
            .post::<RestApi<TradeOrder>>("/api/v5/trade/order", &order_obj)
            .await?)
    }

    // 下单接口

    pub async fn trade_batch_order(&self, order_obj: Vec<OrderRequestInfo>) -> Result<RestApi<TradeOrder>>
    {
        Ok(self
            .post::<RestApi<TradeOrder>>("/api/v5/trade/batch-orders", &order_obj)
            .await?)
    }

    //     获取订单信息
    // 查订单信息
    // GET /api/v5/trade/order
    pub async fn get_trade_order<T>(
        &self,

        inst_id: T,
        ord_id: Option<T>,
        cl_ord_id: Option<T>,
    ) -> Result<RestApi<TradeOrderGet>>
    where
        T: Into<String>,
    {
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        params.insert("instId".into(), inst_id.into());

        if let Some(ord_id) = ord_id {
            params.insert("ordId".into(), ord_id.into());
        }

        if let Some(cl_ord_id) = cl_ord_id {
            params.insert("clOrdId".into(), cl_ord_id.into());
        }

        Ok(self
            .get::<RestApi<TradeOrderGet>>("/api/v5/trade/order", &params)
            .await?)
    }

    //     修改订单
    // 修改当前未成交的挂单
    // POST /api/v5/trade/amend-order

    pub async fn trade_amend_order<T>(
        &self,
        inst_id: T,

        cxl_on_fail: Option<T>,
        ord_id: Option<T>,
        cl_ord_id: Option<T>,

        req_id: Option<T>,
        new_sz: Option<T>,
        new_px: Option<T>,
    ) -> Result<RestApi<TradeAmendOrder>>
    where
        T: Into<String>,
    {
        // let mut params_vec = Vec::new();

        let mut params: BTreeMap<String, String> = BTreeMap::new();

        params.insert("instId".into(), inst_id.into());

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

        Ok(self
            .post::<RestApi<TradeAmendOrder>>("/api/v5/trade/amend-order", &params)
            .await?)
    }
}
