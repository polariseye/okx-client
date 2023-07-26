use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct OkxClient {
    pub debug: bool,
    pub testnet: bool,
    pub account: String,
    pub api_key: String,
    pub secret_key: String,
    pub passphrase: String,
    pub domain: String,
}
impl OkxClient {
    pub fn new(
        debug: bool,
        testnet: bool,
        account: impl Into<String>,
        api_key: impl Into<String>,
        secret_key: impl Into<String>,
        passphrase: impl Into<String>,
        domain: impl Into<String>,
    ) -> Self {
        OkxClient {
            debug: debug,
            testnet: testnet,
            account: account.into(),
            api_key: api_key.into(),
            secret_key: secret_key.into(),
            passphrase: passphrase.into(),
            domain: domain.into(),
        }
    }
}
