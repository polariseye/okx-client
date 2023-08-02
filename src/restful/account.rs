use std::collections::BTreeMap;

use anyhow::Result;

use crate::apikey::OkxAccountClient;
use crate::restful::models::AccountBalance;

use super::models::{AccountPositions, AccountPositionsHistory, AccountSetLeverage, RestApi};

impl OkxAccountClient {
    pub async fn account_balance(&self, ccy_list: Option<Vec<String>>) -> Result<RestApi<AccountBalance>> {
        //  //api/v5/account/balance
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        if let Some(val) = ccy_list {
            params.insert("ccy".into(), val.join(","));
        }

        Ok(self
            .get::<RestApi<AccountBalance>>("/api/v5/account/balance", &params)
            .await?)
    }

    // 检查仓位
    // GET /api/v5/account/positions
    // 查看持仓信息
    // 获取该账户下拥有实际持仓的信息。账户为单向持仓模式会显示净持仓（net），账户为双向持仓模式下会分别返回多头（long）或空头（short）的仓位。按照仓位创建时间倒序排列。
    // GET /api/v5/account/positions

    pub async fn account_positions<T>(
        &self,
        inst_type: Option<T>,
        inst_id: Option<T>,
        pos_id: Option<T>,
        // impl Into<String>
        // pos_side: impl Into<String>,
    ) -> Result<RestApi<AccountPositions>>
    where
        T: Into<String>,
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

        Ok(self
            .get::<RestApi<AccountPositions>>("/api/v5/account/positions", &params)
            .await?)
    }

    // 获取未成交订单列表
    // 获取当前账户下所有未成交订单信息

    // 设置杠杆倍数
    // POST  /api/v5/account/set-leverage
    pub async fn account_set_leverage<T>(
        &self,

        inst_id: Option<T>,
        ccy: Option<T>,
        lever: T,
        mgn_mode: T,
        pos_side: Option<T>,
    ) -> Result<RestApi<AccountSetLeverage>>
    where
        T: Into<String>,
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

        Ok(self
            .post::<RestApi<AccountSetLeverage>>("/api/v5/account/set-leverage", &params)
            .await?)
    }

    // 查看历史持仓信息
    // 获取最近3个月有更新的仓位信息，按照仓位更新时间倒序排列。
    // GET /api/v5/account/positions-history

    pub async fn account_positions_history<T>(
        &self,
        inst_type: Option<T>,
        inst_id: Option<T>,
        mgn_mode: Option<T>,

        ptype: Option<T>,
        pos_id: Option<T>,
        after: Option<T>,
        before: Option<T>,
        limit: Option<T>,
        // impl Into<String>
        // pos_side: impl Into<String>,
    ) -> Result<RestApi<AccountPositionsHistory>>
    where
        T: Into<String>,
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

        Ok(self
            .get::<RestApi<AccountPositionsHistory>>("/api/v5/account/positions-history", &params)
            .await?)
    }
}
