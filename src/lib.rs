pub mod apikey;
pub mod models;
pub mod restful;
pub mod utils;
mod websocket;



#[cfg(test)]
mod test{
    use crate::restful::models::{InstType, OrderRequestInfo, TradeSide};
    use crate::restful::trade::OrdersPendingFilter;
    use super::*;

    #[tokio::test]
    async fn test_client(){
        // https://www.okx.com/docs-v5/zh/#overview-production-trading-services
        /*
            模拟盘交易
            REST：https://www.okx.com
            WebSocket公共频道：wss://wspap.okx.com:8443/ws/v5/public?brokerId=9999
            WebSocket私有频道：wss://wspap.okx.com:8443/ws/v5/private?brokerId=9999
            部分频道：wss://wspap.okx.com:8443/ws/v5/business?brokerId=9999
        */
        /*
            实盘API交易地址如下：

            REST：https://www.okx.com/
            WebSocket公共频道：wss://ws.okx.com:8443/ws/v5/public
            WebSocket私有频道：wss://ws.okx.com:8443/ws/v5/private
            部分频道：wss://ws.okx.com:8443/ws/v5/business
            AWS 地址如下：

            REST：https://aws.okx.com
            WebSocket公共频道：wss://wsaws.okx.com:8443/ws/v5/public
            WebSocket私有频道：wss://wsaws.okx.com:8443/ws/v5/private
            部分频道：wss://wsaws.okx.com:8443/ws/v5/business
        */
        let client_obj = apikey::OkxClient::new(true, true,"454903160406948404","a462e3ed-6866-4ed1-b8e5-8d59126a2a51","B03846A56AEC13A169E3E4C67F11895F","H7ZubBD9FAAffhR!","https://www.okx.com");

        // let balance_result = client_obj.account_balance(Some(vec!["ETH".to_string(), "BTC".to_string()])).await.unwrap();
        // println!("balance result:{:?}", &balance_result);

        // let order = OrderRequestInfo::new_spot_limit_order("ETH-USDT", TradeSide::Buy,"1".into(),"1700".into(), None, None);
        // let resp = client_obj.trade_order(order).await.unwrap();
        // println!("result:{:?}", resp);

        let orders = client_obj.trade_orders_pending(OrdersPendingFilter{
            inst_type: Some(InstType::Spot),
            uly: None,
            inst_family: None,
            inst_id: None,
            ord_type: None,
            state: None,
            after: None,
            before: None,
            limit: None,
        }).await.unwrap();
        println!("result:{:?}", orders);

        // client_obj.trade_cancel_batch_orders(Some("ETH-USDT"), &vec!["604360812546834432".into()],None).await.unwrap();
    }
}