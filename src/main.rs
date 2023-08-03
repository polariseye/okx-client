use std::time::Duration;
use async_trait::async_trait;
use log::LevelFilter;
use crate::restful::models::InstType;
use crate::websocket::{AccountEvent, AccountHandler, EventResponse, OrderBook, OrderBookEvent, OrderBookEventArg, OrderBookMergeHandler, OrderBookSize, OrderBookType, OrderEvent, PublicHandler, PublicWebsocket, TickerEvent, TickerEventArg, TradeEvent, TradeEventArg};

pub mod apikey;
pub mod models;
pub mod restful;
pub mod utils;
mod websocket;

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .with_module_level("tungstenite",LevelFilter::Error)
        .with_module_level("tokio_tungstenite",LevelFilter::Error).init().unwrap();

    pub_test().await;
}

async fn account_test(){
    let account_obj = websocket::AccountWebsocket::start(
        "a462e3ed-6866-4ed1-b8e5-8d59126a2a51",
        "B03846A56AEC13A169E3E4C67F11895F",
        "H7ZubBD9FAAffhR!",
        "wss://wspap.okx.com:8443/ws/v5/private?brokerId=9999").await;
    account_obj.register(TestHandler{});
    account_obj.account_subscribe().await;
    account_obj.order_subscribe(InstType::Spot).await;

    tokio::time::sleep(Duration::from_secs(60)).await;
}
async fn pub_test(){
    let pub_sock = PublicWebsocket::start("wss://wspap.okx.com:8443/ws/v5/public?brokerId=9999").await;
    pub_sock.register(TestHandler{});
    // pub_sock.trade_subscribe("DOT-USDT").await;
    pub_sock.orderbook_subscribe("DOT-USDT", OrderBookSize::Default).await;
    pub_sock.orderbook_merge().register(OrderBookTestHandler{});

    tokio::time::sleep(Duration::from_secs(60)).await;
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

    async fn ticker_event(&self, arg: &TickerEventArg, events: &Vec<TickerEvent>) {
        let event_str = serde_json::to_string(events).unwrap();
        println!("ticker_event:{}", event_str);
    }

    async fn trade_event(&self,  arg:&TradeEventArg, events: &Vec<TradeEvent>) {
        let event_str = serde_json::to_string(events).unwrap();
        println!("trade_event:{}", event_str);
    }

    async fn orderbook_event(&self,arg: &OrderBookEventArg, order_book_type: OrderBookType, size: OrderBookSize, events: &Vec<OrderBookEvent>) {
        let event_str = serde_json::to_string(events).unwrap();
        println!("orderbook_event:{}", event_str);
    }

    async fn on_connected(&self) {
        println!("connected");
    }

    async fn on_disconnected(&self) {
        println!("disconnected");
    }

    async fn handle_response(&self, resp: &EventResponse) {
    }
}

struct OrderBookTestHandler {

}

#[async_trait]
impl OrderBookMergeHandler for OrderBookTestHandler {
    fn id(&self) -> String {
        "test".to_string()
    }

    fn inst_id(&self) -> String {
        "DOT-USDT".to_string()
    }

    async fn on_orderbook_update(&self, orderbook: &OrderBook) {
        let asks = serde_json::to_string(&orderbook.asks).unwrap();
        let bids = serde_json::to_string(&orderbook.bids).unwrap();

        println!("seqId:{} asks:{}", orderbook.seq_id, &asks);
        println!("seqId:{} bids:{}", orderbook.seq_id, &bids);
    }
}