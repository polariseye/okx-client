use std::collections::BTreeMap;

use anyhow::Result;

use crate::apikey::OkxClient;

use super::models::{
    RestApi, TradeAmendOrder, TradeCancelBatchOrders, TradeOrder, TradeOrderGet,
    TradeOrdersHistory, TradeOrdersPending,
};

impl OkxClient {
    // 获取未成交订单列表
    // 获取当前账户下所有未成交订单信息

    pub async fn trade_orders_pending<T>(
        &self,
        inst_type: Option<T>,
        uly: Option<T>,
        inst_family: Option<T>,
        inst_id: Option<T>,
        ord_type: Option<T>,
        state: Option<T>,
        after: Option<T>,
        before: Option<T>,
        limit: Option<T>,
        // impl Into<String>
        // pos_side: impl Into<String>,
    ) -> Result<RestApi<TradeOrdersPending>>
    where
        T: Into<String>,
    {
        //  /api/index/v3/BTC-USD/constituents
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        if let Some(inst_type) = inst_type {
            params.insert("instType".into(), inst_type.into());
        }

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
        if let Some(after) = after {
            params.insert("after".into(), after.into());
        }
        if let Some(before) = before {
            params.insert("before".into(), before.into());
        }
        if let Some(limit) = limit {
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

    pub async fn trade_order<T>(
        &self,
        inst_id: T,
        td_mode: T,
        ccy: Option<T>,
        cl_ord_id: Option<T>,
        tag: Option<T>,
        side: T,
        pos_side: Option<T>,
        ord_type: T,
        sz: T,
        px: Option<T>,
        reduce_only: Option<bool>,
        tgt_ccy: Option<T>,
        ban_amend: Option<bool>,
        tp_trigger_px: Option<T>,
        tp_ord_px: Option<T>,
        sl_trigger_px: Option<T>,
        sl_ord_px: Option<T>,
        tp_trigger_px_type: Option<T>,
        sl_trigger_px_type: Option<T>,
        quick_mgn_type: Option<T>,
    ) -> Result<RestApi<TradeOrder>>
    where
        T: Into<String>,
    {
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        params.insert("instId".into(), inst_id.into());

        // 保证金模式：isolated：逐仓 ；cross：全仓
        params.insert("tdMode".into(), td_mode.into());
        // params.insert("clOrdId".into(), "swap888".into());
        params.insert("side".into(), side.into());

        params.insert("ordType".into(), ord_type.into());
        params.insert("sz".into(), sz.into());

        if let Some(ccy) = ccy {
            params.insert("ccy".into(), ccy.into());
        }

        if let Some(cl_ord_id) = cl_ord_id {
            params.insert("clOrdId".into(), cl_ord_id.into());
        }

        // 支持苦逼的brokerCode !self.debug ||
        if !self.testnet {
            params.insert("tag".into(), "203a44225069BCDE".into());
        } else {
            if let Some(tag) = tag {
                params.insert("tag".into(), tag.into());
            }
        }

        if let Some(pos_side) = pos_side {
            params.insert("posSide".into(), pos_side.into());
        }

        if let Some(px) = px {
            params.insert("px".into(), px.into());
        }

        if let Some(reduce_only) = reduce_only {
            params.insert("reduceOnly".into(), reduce_only.to_string());
        }

        if let Some(tgt_ccy) = tgt_ccy {
            params.insert("tgtCcy".into(), tgt_ccy.into());
        }

        if let Some(ban_amend) = ban_amend {
            params.insert("banAmend".into(), ban_amend.to_string());
        }

        if let Some(tp_trigger_px) = tp_trigger_px {
            params.insert("tpTriggerPx".into(), tp_trigger_px.into());
        }

        if let Some(tp_ord_px) = tp_ord_px {
            params.insert("tpOrdPx".into(), tp_ord_px.into());
        }
        if let Some(sl_trigger_px) = sl_trigger_px {
            params.insert("slTriggerPx".into(), sl_trigger_px.into());
        }
        if let Some(sl_ord_px) = sl_ord_px {
            params.insert("slOrdPx".into(), sl_ord_px.into());
        }

        if let Some(tp_trigger_px_type) = tp_trigger_px_type {
            params.insert("tpTriggerPxType".into(), tp_trigger_px_type.into());
        }
        if let Some(sl_trigger_px_type) = sl_trigger_px_type {
            params.insert("slTriggerPxType".into(), sl_trigger_px_type.into());
        }
        if let Some(quick_mgn_type) = quick_mgn_type {
            params.insert("quickMgnType".into(), quick_mgn_type.into());
        }

        Ok(self
            .post::<RestApi<TradeOrder>>("/api/v5/trade/order", &params)
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
