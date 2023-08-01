use crate::utils;
use async_trait::async_trait;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::string::String;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_tungstenite::tungstenite::handshake::client::Response;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::error::ProtocolError;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ConnState {
    Connecting,
    Connected,
    Closing,
    Closed,
}

enum OkxMessage {
    Message(String),
    Close,
}

pub struct WebsocketConn<THandler> {
    handler: Weak<THandler>,
    remote_url: String,
    send_chan: Sender<OkxMessage>,
    state: RwLock<ConnState>,
    last_pong_time: AtomicI64,
}

impl<THandler: Handler + 'static> WebsocketConn<THandler> {
    pub async fn start(
        handler: Weak<THandler>,
        remote_url: impl Into<String>,
    ) -> Arc<WebsocketConn<THandler>> {
        let (send_sender, send_receiver) = tokio::sync::mpsc::channel::<OkxMessage>(256);

        let result = Arc::new(Self {
            handler,
            remote_url: remote_url.into(),
            send_chan: send_sender,
            state: RwLock::new(ConnState::Connecting),
            last_pong_time: AtomicI64::new(0),
        });

        let cloned = result.clone();
        tokio::spawn(Self::connect(cloned, send_receiver));

        result
    }

    fn handler(&self) -> Option<Arc<THandler>> {
        self.handler.upgrade()
    }

    async fn connect(
        conn_obj: Arc<WebsocketConn<THandler>>,
        mut send_receiver: Receiver<OkxMessage>,
    ) {
        loop {
            conn_obj.set_state(ConnState::Connecting);
            let mut conn;
            match tokio_tungstenite::connect_async(&conn_obj.remote_url).await {
                Err(err) => {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    error!(
                        "connect to okx({}) error:{}",
                        &conn_obj.remote_url,
                        err.to_string()
                    );
                    continue;
                }
                Ok(val) => {
                    conn = val.0;
                }
            }
            let (mut sender, receiver) = conn.split();

            // 开启处理协程
            let sender_conn = conn_obj.clone();
            sender_conn.last_pong_time.store(utils::get_unix(),  Ordering::SeqCst);
            let is_closed = Arc::new(AtomicBool::new(false));
            tokio::spawn(async move {
                sender_conn.receive_tokio(receiver).await;
            });

            let on_conn_handle_conn = conn_obj.clone();
            let handler = on_conn_handle_conn.handler();
            if handler.is_none() {
                // 如果处理对象都已经不存在了，则应该结束
                break;
            }
            let receive_wait_handle = tokio::spawn(async move {
                if let Some(handler) = handler {
                    handler.on_connected().await
                } else {
                    warn!("on connected, but handler no exist");
                }
            });

            conn_obj.set_state(ConnState::Connected);
            let is_close_by_user = conn_obj
                .send_tokio(&mut sender, &mut send_receiver)
                .await;
            let _ = sender.close().await;
            let _ = receive_wait_handle.await;
            let handler = on_conn_handle_conn.handler();
            if handler.is_none() {
                // 如果处理对象都已经不存在了，则应该结束
                break;
            }
            tokio::spawn(async move {
                if let Some(handler) = handler {
                    handler.on_disconnected().await;
                } else {
                    warn!("on disconnected, but handler no exist");
                }
            });
            if is_close_by_user {
                conn_obj.set_state(ConnState::Closed);
                break;
            }
        }

        send_receiver.close();
    }

    async fn send_tokio(
        &self,
        sender: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        send_receiver: &mut Receiver<OkxMessage>,
    ) -> bool {
        let mut ticker = tokio::time::interval(Duration::from_secs(5));

        loop {
            select! {
                item = send_receiver.recv() => {
                    match item {
                        Some(to_send_item) => {
                            match to_send_item {
                                OkxMessage::Message(val) => {
                                    if let Err(err) = sender.send(Message::Text(val)).await {
                                        // 错误处理
                                        error!("send message error:{}", err.to_string());
                                        if self.handle_err(&err) {
                                            return false;
                                        }
                                    }
                                },
                                OkxMessage::Close => {
                                    info!("start close conn");
                                    return true;
                                }
                            }
                        },
                        None => {
                            info!("send channel closed");
                            return true;
                        }
                    }
                },
                _ = ticker.tick() => {
                    if let Err(err) = sender.send(Message::Text("ping".to_string())).await {
                        warn!("send message error:{}", err.to_string());
                        if self.handle_err(&err) {
                            return false;
                        }
                    }
                    let now = chrono::Utc::now().timestamp();
                    if now > self.last_pong_time.load(Ordering::SeqCst) +30*1000 {
                        // 接收pong超时
                        return false;
                    }
                }
            }
        }
    }

    fn handle_err(&self, err: &Error) -> bool {
        match err {
            Error::ConnectionClosed => true,
            Error::AlreadyClosed => true,
            Error::Io(_) => true,
            Error::Tls(_) => false,
            Error::Capacity(_) => false,
            Error::Protocol(err) => {
                match err {
                    ProtocolError::SendAfterClosing => true,
                    ProtocolError::ReceivedAfterClosing => true,
                    _ => false
                }
            },
            Error::SendQueueFull(_) => false,
            Error::Utf8 => false,
            Error::Url(_) => false,
            Error::Http(_) => false,
            Error::HttpFormat(_) => false,
        }
    }

    async fn receive_tokio(
        &self,
        mut receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) {

        loop {
            select! {
                message = receiver.next() => {
                    if let Some(message) = message {
                        match message {
                            Ok(val) => {
                                match val {
                                    Message::Text(val) => {
                                        if val == "pong" {
                                            // pong消息处理
                                            self.last_pong_time.store(utils::get_unix(), Ordering::SeqCst);
                                        } else {
                                            if let Err(err) = self.handle_message(val).await {
                                                error!("handle message error. {}", err.to_string());
                                            }
                                        }
                                    },
                                    Message::Binary(val) => {

                                    },
                                    Message::Close(val) => {
                                        return;
                                    },
                                    _ => {

                                    }
                                }
                            },
                            Err(err) => {
                                warn!("receive_tokio error:{}", err.to_string());
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    async fn handle_message(&self, message: String) -> anyhow::Result<()> {
        let event_resp: EventResponse =
            serde_json::from_str(&message).map_err(|err| anyhow::anyhow!(err))?;

        if let Some(handler) = self.handler() {
            handler.handle_response(event_resp).await;
        }

        Ok(())
    }

    pub fn state(&self) -> ConnState {
        self.state.read().unwrap().clone()
    }

    fn set_state(&self, state: ConnState) {
        let mut writer = self.state.write().unwrap();
        *writer = state
    }

    pub async fn send(&self, req: impl Serialize) -> anyhow::Result<()> {
        if self.state() != ConnState::Connected {
            return Err(anyhow::anyhow!("not connected"));
        }
        match serde_json::to_string(&req) {
            Ok(val) => {
                if let Err(err) = self.send_chan.send(OkxMessage::Message(val)).await {
                    error!("send message error:{}", err.to_string());
                    return Err(anyhow::anyhow!(err));
                }

                Ok(())
            }
            Err(err) => {
                error!("unmarshal message error:{}", err.to_string());
                return Err(anyhow::anyhow!(err));
            }
        }
    }

    pub async fn send_request(&self, op: &str, req: impl Serialize) -> anyhow::Result<()> {
        let req_val;
        match serde_json::to_value(&req) {
            Ok(val) => req_val = val,
            Err(err) => {
                error!("unmarshal request error:{}", err.to_string());
                return Err(anyhow::anyhow!(err));
            }
        }

        self.send(&WebsocketRequest {
            op: "login".to_string(),
            args: vec![req_val],
        })
        .await
    }

    pub async fn close(&self) -> anyhow::Result<()> {
        self.send_chan
            .send(OkxMessage::Close)
            .await
            .map_err(|err| anyhow::anyhow!(err))
    }
}

#[derive(Serialize, Debug)]
pub struct WebsocketRequest {
    pub op: String,
    pub args: Vec<serde_json::Value>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct EventResponse {
    #[serde(default = "String::default")]
    pub event: String,
    pub arg: Option<serde_json::Value>,
    pub data: Option<serde_json::Value>,
    #[serde(default = "String::default")]
    pub msg: String,
    #[serde(default = "zero_code")]
    pub code: String,
}

fn zero_code() -> String {
    "0".into()
}

#[async_trait]
pub trait Handler: Send + Sync {
    async fn on_connected(&self);
    async fn on_disconnected(&self);
    async fn handle_response(&self, resp: EventResponse);
}
