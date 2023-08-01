use std::time::Duration;
use async_trait::async_trait;
use log::LevelFilter;
use crate::restful::models::InstType;
use crate::websocket::{AccountEvent, AccountHandler, OrderEvent};

pub mod apikey;
pub mod models;
pub mod restful;
pub mod utils;
mod websocket;

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .with_module_level("tungstenite",LevelFilter::Error)
        .with_module_level("tokio_tungstenite",LevelFilter::Error).init().unwrap();

    let account_obj = websocket::AccountWebsocket::start(TestHandler{}, "a462e3ed-6866-4ed1-b8e5-8d59126a2a51", "B03846A56AEC13A169E3E4C67F11895F", "wss://wspap.okx.com:8443/ws/v5/private?brokerId=9999").await;
    account_obj.account_subscribe().await;
    account_obj.order_subscribe(InstType::Spot).await;
    tokio::time::sleep(Duration::from_secs(20)).await;
}

struct TestHandler {}
#[async_trait]
impl AccountHandler for TestHandler {
    async fn account_event(&self, events: Vec<AccountEvent>) {
        let event_str = serde_json::to_string(&events).unwrap();
        println!("account event:{}", event_str);
    }

    async fn order_event(&self, events: Vec<OrderEvent>) {
        let event_str = serde_json::to_string(&events).unwrap();
        println!("order event:{}", event_str);
    }
}