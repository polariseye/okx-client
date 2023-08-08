use std::sync::Arc;
use serde::Deserialize;
use crate::utils::request_limit::LimitMgr;
use crate::websocket::{AccountWebsocket, PublicWebsocket};

#[derive(Debug, Clone)]
pub struct OkxPublicClient {
    pub base_config: OkxConfig,
    limit_mgr: Arc<LimitMgr>,
}

impl OkxPublicClient {
    pub fn new(base_config: OkxConfig) -> Self {
        Self{
            base_config,
            limit_mgr: Arc::new(LimitMgr::new()),
        }
    }

    pub async fn start_websocket(&self) -> Arc<PublicWebsocket> {
        PublicWebsocket::start(&self.base_config.pub_websocket_domain).await
    }

    pub(crate) fn limit_mgr(&self) -> &LimitMgr {
        &self.limit_mgr
    }
}

#[derive(Debug)]
pub struct OkxAccountClient {
    pub api_key: String,
    pub secret_key: String,
    pub passphrase: String,
    pub base_config: OkxConfig,
    limit_mgr: LimitMgr,
}

impl OkxAccountClient {
    pub fn new(
        base_config: OkxConfig,
        api_key: impl Into<String>,
        secret_key: impl Into<String>,
        passphrase: impl Into<String>,
    ) -> Self {
        OkxAccountClient {
            base_config,
            api_key: api_key.into(),
            secret_key: secret_key.into(),
            passphrase: passphrase.into(),
            limit_mgr: LimitMgr::new(),
        }
    }

    pub async fn start_websocket(&self) -> Arc<AccountWebsocket> {
        AccountWebsocket::start(&self.api_key, &self.secret_key, &self.passphrase, &self.base_config.private_websocket_domain).await
    }

    pub(crate) fn limit_mgr(&self) -> &LimitMgr {
        &self.limit_mgr
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OkxConfig {
    pub testnet: bool,
    pub rest_domain: String,
    pub pub_websocket_domain: String,
    pub private_websocket_domain: String,
    pub business_domain: String,
} 

#[allow(unused)]
pub fn mainnet_config() -> OkxConfig{
    OkxConfig{
        testnet: false,
        rest_domain: "https://www.okx.com/".to_string(),
        pub_websocket_domain: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
        private_websocket_domain: "wss://ws.okx.com:8443/ws/v5/private".to_string(),
        business_domain: "wss://ws.okx.com:8443/ws/v5/business".to_string(),
    }
}

#[allow(unused)]
pub fn aws_mainnet_config()-> OkxConfig{
    OkxConfig {
        testnet: false,
        rest_domain: "https://aws.okx.com".to_string(),
        pub_websocket_domain: "wss://wsaws.okx.com:8443/ws/v5/public".to_string(),
        private_websocket_domain: "wss://wsaws.okx.com:8443/ws/v5/private".to_string(),
        business_domain: "wss://wsaws.okx.com:8443/ws/v5/business".to_string(),
    }
}

#[allow(unused)]
pub fn testnet_config() -> OkxConfig {
    OkxConfig {
        testnet: true,
        rest_domain: "https://www.okx.com".to_string(),
        pub_websocket_domain: "wss://wspap.okx.com:8443/ws/v5/public?brokerId=9999".to_string(),
        private_websocket_domain: "wss://wspap.okx.com:8443/ws/v5/private?brokerId=9999".to_string(),
        business_domain: "wss://wspap.okx.com:8443/ws/v5/business?brokerId=9999".to_string(),
    }
}

impl OkxConfig {
    pub fn create_account_client(self,
                                 api_key: impl Into<String>,
                                 secret_key: impl Into<String>,
                                 passphrase: impl Into<String>,) -> OkxAccountClient {
        OkxAccountClient::new(self, api_key, secret_key, passphrase)
    }

    pub fn create_pub_client(self) -> OkxPublicClient {
        OkxPublicClient::new(self)
    }
}