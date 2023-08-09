use std::time::Duration;
use async_trait::async_trait;
use log::LevelFilter;
use okx_client::*;
use okx_client::websocket::*;

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .with_module_level("tungstenite",LevelFilter::Error)
        .with_module_level("tokio_tungstenite",LevelFilter::Error).init().unwrap();

    // let pub_client = testnet_config().create_pub_client();
    // let insts = pub_client.public_instruments(InstType::Spot, Option::<String>::None, Option::<String>::None, Option::<String>::None).await.unwrap();
    // println!("result:{:?}", &insts);

    account_test().await;
}

async fn account_test(){
    let rest_account = testnet_config().create_account_client(
        "a462e3ed-6866-4ed1-b8e5-8d59126a2a51",
        "B03846A56AEC13A169E3E4C67F11895F",
        "H7ZubBD9FAAffhR!");
    // rest_account.account_config().await.unwrap();
    // rest_account.trade_orders_pending(OrdersPendingFilter{
    //     inst_type: Some(InstType::Spot),
    //     uly: None,
    //     inst_family: None,
    //     inst_id: None,
    //     ord_type: None,
    //     state: None,
    //     after: None,
    //     before: None,
    //     limit: None,
    // }).await.unwrap();
    let account_obj = rest_account.start_websocket().await;
    account_obj.register(TestHandler{});
    account_obj.account_subscribe().await;
    account_obj.order_subscribe(InstType::Spot).await;

    tokio::time::sleep(Duration::from_secs(60)).await;
}
async fn pub_test(){
    let pub_sock = testnet_config().create_pub_client().start_websocket().await;
    pub_sock.register(TestHandler{});
    // pub_sock.trade_subscribe("DOT-USDT").await;
    pub_sock.orderbook_subscribe("DOT-USDT", OrderBookSize::Default).await;
    pub_sock.orderbook_merge().register(OrderBookTestHandler{});

    tokio::time::sleep(Duration::from_secs(60*10)).await;
}

struct TestHandler {}

#[async_trait]
impl AccountHandler for TestHandler {
    fn id(&self) -> String {
        "test handler".into()
    }

    async fn account_event(&self, events: &Vec<AccountEvent>) {
        let event_str = serde_json::to_string(events).unwrap();
        println!("account event:{}", event_str);
    }

    async fn order_event(&self, events: &Vec<OrderEvent>) {
        let event_str = serde_json::to_string(events).unwrap();
        println!("order event:{}", event_str);
    }
}

#[async_trait]
impl PublicHandler for TestHandler {
    fn id(&self) -> String {
        AccountHandler::id(self)
    }

    async fn ticker_event(&self, _arg: &TickerEventArg, events: &Vec<TickerEvent>) {
        let event_str = serde_json::to_string(events).unwrap();
        println!("ticker_event:{}", event_str);
    }

    async fn trade_event(&self,  _arg:&TradeEventArg, events: &Vec<TradeEvent>) {
        let event_str = serde_json::to_string(events).unwrap();
        println!("trade_event:{}", event_str);
    }

    async fn orderbook_event(&self,_arg: &OrderBookEventArg, _order_book_type: OrderBookType, _size: OrderBookSize, _events: &Vec<OrderBookEvent>) {
        // let event_str = serde_json::to_string(events).unwrap();
        // println!("orderbook_event:{}", event_str);
    }

    async fn on_connected(&self) {
        println!("connected");
    }

    async fn on_disconnected(&self) {
        println!("disconnected");
    }

    async fn handle_response(&self, _resp: &EventResponse) {
    }
}

struct OrderBookTestHandler {

}

#[async_trait]
impl OrderBookMergeHandler for OrderBookTestHandler {
    fn id(&self) -> String {
        "test".to_string()
    }

    async fn on_orderbook_update(&self, orderbook: &OrderBook) {
        let asks = serde_json::to_string(&orderbook.asks).unwrap();
        let bids = serde_json::to_string(&orderbook.bids).unwrap();

        println!("seqId:{} len:{} asks:{}", orderbook.seq_id, orderbook.asks.len(), &asks);
        println!("seqId:{} len:{} bids:{}", orderbook.seq_id, orderbook.bids.len(), &bids);
    }
}