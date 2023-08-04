use std::collections::BTreeMap;

use anyhow::Result;

use super::models::{Instrument, RestApi};
use crate::apikey::OkxPublicClient;

impl OkxPublicClient {
    //     获取交易产品基础信息
    // 获取所有可交易产品的信息列表。
    // GET /api/v5/public/instruments
    pub async fn public_instruments<T>(
        &self,
        inst_type: T,
        uly: Option<T>,
        inst_family: Option<T>,
        inst_id: Option<T>,
        // impl Into<String>
        // pos_side: impl Into<String>,
    ) -> Result<RestApi<Instrument>>
    where
        T: Into<String>,
    {
        //  /api/index/v3/BTC-USD/constituents
        let mut params: BTreeMap<String, String> = BTreeMap::new();

        if let Some(uly) = uly {
            params.insert("uly".into(), uly.into());
        }

        if let Some(inst_family) = inst_family {
            params.insert("instFamily".into(), inst_family.into());
        }

        if let Some(inst_id) = inst_id {
            params.insert("instId".into(), inst_id.into());
        }

        params.insert("instType".into(), inst_type.into());

        Ok(self
            .get::<RestApi<Instrument>>("/api/v5/public/instruments", &params)
            .await?)
    }
}

#[cfg(test)]
mod test{
    use crate::InstType;

    #[tokio::test]
    pub async fn test_instrument() {
        let pub_client = crate::testnet_config().create_pub_client();
        let result = pub_client.public_instruments(InstType::Spot, None, None, None).await.unwrap();
        println!("result:{:?}", result);
    }
}
