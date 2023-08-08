use std::num::ParseIntError;
use thiserror::Error;

pub type Result<T> = core::result::Result<T, OkxError>;

#[derive(Error, Debug)]
pub enum OkxError {
    #[error("rate limit")]
    RateLimit,
    #[error("serde error: {0:?}")]
    SerdeError(#[from]serde_json::Error),
    #[error("reqwest error: {0:?}")]
    ReqwestError(#[from]reqwest::Error),
    #[error("decimal convert error: {0:?}")]
    DecimalError(#[from]rust_decimal::Error),
    #[error("int convert error: {0:?}")]
    ParseIntError(#[from]ParseIntError),
    #[error("websocket not connected")]
    NotConnect,
    #[error("okx response error. code:{code} message:{message}")]
    RemoteError{ code: i32, message: String},
}