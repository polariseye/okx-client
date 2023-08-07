use std::collections::BTreeMap;
use anyhow::Result;
use super::models::{MarketBooks, MarketTicker, MarketTickers, RestApi, Trade};
use crate::apikey::OkxPublicClient;

impl OkxPublicClient {
    // 获取所有产品行情信息
    // GET /api/v5/market/tickers

    pub async fn market_tickers(
        &self,
        inst_type: impl Into<String>,
        uly: Option<impl Into<String>>,
        inst_family: Option<impl Into<String>>,
    ) -> Result<RestApi<MarketTickers>>
    {
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        if let Some(uly) = uly {
            params.insert("uly".into(), uly.into());
        }

        if let Some(inst_family) = inst_family {
            params.insert("instFamily".into(), inst_family.into());
        }

        params.insert("instType".into(), inst_type.into());

        Ok(self
            .get::<RestApi<MarketTickers>>("/api/v5/market/tickers", &params)
            .await?)
    }

    // 获取单个产品行情信息
    // 获取产品行情信息
    // GET /api/v5/market/ticker
    pub async fn market_ticker(&self, inst_id: impl Into<String>) -> Result<RestApi<MarketTicker>>
    {
        //  /api/index/v3/BTC-USD/constituents
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        params.insert("instId".into(), inst_id.into());

        Ok(self
            .get::<RestApi<MarketTicker>>("/api/v5/market/ticker", &params)
            .await?)
    }

    /// 获取交易产品公共成交数据
    /// GET /api/v5/market/trades
    pub async fn market_trades(&self, inst_id: impl Into<String>, limit: usize) -> Result<RestApi<Trade>>{
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        // /api/v5/market/trades
        params.insert("instId".into(), inst_id.into());
        params.insert("limit".into(), limit.to_string());

        Ok(self
            .get::<RestApi<Trade>>("/api/v5/market/trades", &params)
            .await?)
    }

    // 获取深度

    // api/v5/market/books

    pub async fn market_books(&self, inst_id: impl Into<String>, sz: Option<impl Into<String>>) -> Result<RestApi<MarketBooks>>
    {
        //  /api/index/v3/BTC-USD/constituents
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        params.insert("instId".into(), inst_id.into());

        if let Some(sz) = sz {
            params.insert("sz".into(), sz.into());
        }

        // get_json_value
        // let aaa = self.get_json_value("/api/v5/market/books", &params).await?;
        // println!("aaa:{:?}", aaa);

        // println!(
        //     "bbb:{:?}",
        //     serde_json::from_value::<RestApi<MarketBooks>>(aaa).unwrap()
        // );
        Ok(self
            .get::<RestApi<MarketBooks>>("/api/v5/market/books", &params)
            .await?)
    }
}
