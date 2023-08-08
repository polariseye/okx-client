use crate::utils;
use async_trait::async_trait;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use log::{error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::string::String;
use std::sync::{Arc, RwLock, Weak};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::error::ProtocolError;
use crate::okx_error::*;

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
}

enum PongMessage {
    Close,
    Pong,
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
            let conn;
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
            let (pong_sender, pong_receiver) = tokio::sync::mpsc::channel::<PongMessage>(16);
            tokio::spawn(async move {
                sender_conn.receive_tokio(receiver, pong_sender).await;
            });

            // 清空消息队列, 因为这些消息是之前的老消息。
            loop {
                if send_receiver.try_recv().is_err() {
                    break
                }
            }

            // 开启消息发送逻辑
            conn_obj.set_state(ConnState::Connected);
            let on_conn_handle_conn = conn_obj.clone();
            let handler = on_conn_handle_conn.handler();
            if handler.is_none() {
                // 如果处理对象都已经不存在了，则应该结束
                warn!("not found handler. will stop all");
                break;
            }
            let receive_wait_handle = tokio::spawn(async move {
                if let Some(handler) = handler {
                    handler.on_connected().await
                } else {
                    warn!("on connected, but handler no exist");
                }
            });

            let is_close_by_user = conn_obj
                .send_tokio(&mut sender, &mut send_receiver, pong_receiver)
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

    // 返回true: 退出重连, 返回false: 进行重连
    async fn send_tokio(
        &self,
        sender: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        send_receiver: &mut Receiver<OkxMessage>,
        mut pong_receiver: Receiver<PongMessage>,
    ) -> bool {
        let mut ticker = tokio::time::interval(Duration::from_secs(5));
        let mut last_pong_time = utils::get_unix();

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
                item = pong_receiver.recv() => {
                    match item {
                        Some(item) => {
                            match item {
                                PongMessage::Pong => {
                                    last_pong_time = utils::get_unix();
                                },
                                PongMessage::Close => {
                                    return false;
                                }
                            }
                        },
                        None => {
                            info!("pong receiver closed");
                            return false;
                        }
                    }
                }
                _ = ticker.tick() => {
                    if let Err(err) = sender.send(Message::Text("ping".to_string())).await {
                        warn!("send message error:{}", err.to_string());
                        if self.handle_err(&err) {
                            return false;
                        }
                    }
                    let now = chrono::Utc::now().timestamp();
                    if now > last_pong_time+30*1000 {
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
        pong_sender: Sender<PongMessage>,
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
                                            let _ = pong_sender.send(PongMessage::Pong).await;
                                        } else {
                                            trace!("received message:{}", &val);
                                            if let Err(err) = self.handle_message(val).await {
                                                error!("handle message error. {}", err.to_string());
                                            }
                                        }
                                    },
                                    Message::Binary(_val) => {

                                    },
                                    Message::Close(_val) => {
                                        let _ = pong_sender.send(PongMessage::Close).await;
                                        return;
                                    },
                                    _ => {

                                    }
                                }
                            },
                            Err(err) => {
                                warn!("receive_tokio error:{}", err.to_string());
                                let _ = pong_sender.send(PongMessage::Close).await;
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    async fn handle_message(&self, message: String) -> Result<()> {
        let event_resp: EventResponse =
            serde_json::from_str(&message).map_err(|err| OkxError::SerdeError(err))?;

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

    pub async fn send(&self, req: impl Serialize) -> Result<()> {
        if self.state() != ConnState::Connected {
            return Err(OkxError::NotConnect);
        }
        match serde_json::to_string(&req) {
            Ok(val) => {
                if let Err(err) = self.send_chan.send(OkxMessage::Message(val)).await {
                    error!("send message error:{}", err.to_string());
                    return Err(OkxError::NotConnect);
                }

                Ok(())
            }
            Err(err) => {
                error!("unmarshal message error:{}", err.to_string());
                return Err(err.into());
            }
        }
    }

    pub async fn send_request(&self, op: &str, req: impl Serialize) -> Result<()> {
        let req_val;
        match serde_json::to_value(&req) {
            Ok(val) => req_val = val,
            Err(err) => {
                error!("unmarshal request error:{}", err.to_string());
                return Err(err.into());
            }
        }

        self.send(&WebsocketRequest {
            op: op.to_string(),
            args: vec![req_val],
        })
        .await
    }

    pub async fn close(&self) -> Result<()> {
        self.send_chan
            .send(OkxMessage::Close)
            .await
            .map_err(|_err| OkxError::NotConnect)
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
    pub action: String,
    #[serde(default = "String::default")]
    pub msg: String,
    #[serde(default = "zero_code")]
    pub code: String,
}

impl EventResponse {
    pub fn channel(&self) -> Option<String> {
        if let Some(arg) = &self.arg {
            if let Some(arg_detail) = arg.as_object() {
                if let Some(val) = arg_detail.get("channel") {
                    return Some(val.as_str().unwrap().to_string());
                }
            }
        }

        None
    }
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
