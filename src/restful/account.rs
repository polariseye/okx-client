use std::collections::BTreeMap;
use crate::okx_error::*;
use crate::apikey::OkxAccountClient;
use crate::InstType;
use crate::restful::models::AccountBalance;

use super::models::{AccountPositions, AccountPositionsHistory, AccountSetLeverage, RestApi};

impl OkxAccountClient {
    pub async fn account_balance(&self, ccy_list: Option<Vec<String>>) -> Result<Vec<AccountBalance>> {
        //  /api/v5/account/balance
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        if let Some(val) = ccy_list {
            params.insert("ccy".into(), val.join(","));
        }

        self
            .get::<RestApi<AccountBalance>>("/api/v5/account/balance", &params)
            .await?.to_result()
    }

    // 检查仓位
    // GET /api/v5/account/positions
    // 查看持仓信息
    // 获取该账户下拥有实际持仓的信息。账户为单向持仓模式会显示净持仓（net），账户为双向持仓模式下会分别返回多头（long）或空头（short）的仓位。按照仓位创建时间倒序排列。
    // GET /api/v5/account/positions

    pub async fn account_positions(
        &self,
        inst_type: Option<InstType>,
        inst_id: Option<impl Into<String>>,
        pos_id: Option<impl Into<String>>,
        // impl Into<String>
        // pos_side: impl Into<String>,
    ) -> Result<Vec<AccountPositions>>
    {
        //  /api/index/v3/BTC-USD/constituents
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        if let Some(inst_type) = inst_type {
            params.insert("instType".into(), inst_type.into());
        }

        if let Some(inst_id) = inst_id {
            params.insert("instId".into(), inst_id.into());
        }

        if let Some(pos_id) = pos_id {
            params.insert("posId".into(), pos_id.into());
        }

        self
            .get::<RestApi<AccountPositions>>("/api/v5/account/positions", &params)
            .await?.to_result()
    }

    // 获取未成交订单列表
    // 获取当前账户下所有未成交订单信息

    // 设置杠杆倍数
    // POST  /api/v5/account/set-leverage
    pub async fn account_set_leverage(
        &self,
        inst_id: Option<impl Into<String>>,
        ccy: Option<impl Into<String>>,
        lever: impl Into<String>,
        mgn_mode: impl Into<String>,
        pos_side: Option<impl Into<String>>,
    ) -> Result<AccountSetLeverage>
    {
        //  /api/index/v3/BTC-USD/constituents
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        if let Some(inst_id) = inst_id {
            params.insert("instId".into(), inst_id.into());
        }

        if let Some(ccy) = ccy {
            params.insert("ccy".into(), ccy.into());
        }

        if let Some(pos_side) = pos_side {
            params.insert("posSide".into(), pos_side.into());
        }

        params.insert("lever".into(), lever.into());
        params.insert("mgnMode".into(), mgn_mode.into());

        self
            .post::<RestApi<AccountSetLeverage>>("/api/v5/account/set-leverage", &params)
            .await?.to_result_one()
    }

    // 查看历史持仓信息
    // 获取最近3个月有更新的仓位信息，按照仓位更新时间倒序排列。
    // GET /api/v5/account/positions-history

    pub async fn account_positions_history(
        &self,
        inst_type: Option<impl Into<String>>,
        inst_id: Option<impl Into<String>>,
        mgn_mode: Option<impl Into<String>>,

        ptype: Option<impl Into<String>>,
        pos_id: Option<impl Into<String>>,
        after: Option<impl Into<String>>,
        before: Option<impl Into<String>>,
        limit: Option<impl Into<String>>,
        // impl Into<String>
        // pos_side: impl Into<String>,
    ) -> Result<Vec<AccountPositionsHistory>>
    {
        //  /api/index/v3/BTC-USD/constituents
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        if let Some(inst_type) = inst_type {
            params.insert("instType".into(), inst_type.into());
        }

        if let Some(inst_id) = inst_id {
            params.insert("instId".into(), inst_id.into());
        }

        if let Some(mgn_mode) = mgn_mode {
            params.insert("mgnMode".into(), mgn_mode.into());
        }

        if let Some(ptype) = ptype {
            params.insert("type".into(), ptype.into());
        }
        if let Some(pos_id) = pos_id {
            params.insert("posId".into(), pos_id.into());
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

        self
            .get::<RestApi<AccountPositionsHistory>>("/api/v5/account/positions-history", &params)
            .await?.to_result()
    }
}
