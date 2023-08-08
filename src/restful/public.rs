use std::collections::BTreeMap;
use crate::api_enum::APiEnum;
use crate::okx_error::*;
use super::models::{Instrument, RestApi};
use crate::apikey::OkxPublicClient;
use crate::InstType;

impl OkxPublicClient {
    //     获取交易产品基础信息
    // 获取所有可交易产品的信息列表。
    // GET /api/v5/public/instruments
    pub async fn public_instruments(
        &self,
        inst_type: InstType,
        uly: Option<impl Into<String>>,
        inst_family: Option<impl Into<String>>,
        inst_id: Option<impl Into<String>>,
        // impl Into<String>
        // pos_side: impl Into<String>,
    ) -> Result<Vec<Instrument>>
    {
        //  /api/index/v3/BTC-USD/constituents
        self.limit_mgr().check_limit_with_inst_type(APiEnum::PublicInstruments as u32, inst_type, 1, 20, 2)?;
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

        self
            .get::<RestApi<Instrument>>("/api/v5/public/instruments", &params)
            .await?.to_result()
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
