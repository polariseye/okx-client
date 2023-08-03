use std::collections::BTreeMap;
use std::fmt::Debug;

use anyhow::Result;
use chrono::Utc;

use http::{HeaderMap, HeaderValue};
use log::debug;
use ring::hmac;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::apikey::{OkxAccountClient, OkxPublicClient};

impl OkxAccountClient {
    pub async fn get<T>(
        &self,
        request_path: &str,
        parameters: &BTreeMap<String, String>,
    ) -> Result<T>
    where
        T: DeserializeOwned + std::fmt::Debug,
    {
        // # 获取本地时间
        let timestamp = self.get_timestamp();

        let mut get_url_params: String = String::from(request_path);

        // 有数据就格式化参数
        if parameters.len() != 0 {
            get_url_params = format!("{}?{}", request_path, self.parse_params_to_str(parameters));
        }
        // println!("url {}",get_url_params);
        // OK-ACCESS-SIGN的请求头是对timestamp + method + requestPath + body字符串(+表示字符串连接)，以及SecretKey

        // let message = format!("{}GET{}{}", timestamp, get_url_params, &self.secret_key);
        let message = format!("{}GET{}", timestamp, get_url_params);

        // println!("{:?}", message);

        let sign = self.sign(&message);

        let headers = self.create_header(&sign, &timestamp);

        let client = reqwest::Client::new();

        debug!(
            "[*] Debug:\nUrl:{}\nparameters:{:?}",
            format!("{}{}", self.base_config.rest_domain, request_path),
            parameters
        );

        let res = client
            .get(format!("{}{}", self.base_config.rest_domain, get_url_params))
            .headers(headers)
            .send()
            .await?
            .text()
            .await?;

        debug!("[*] Response {:#?}", res);

        Ok(serde_json::from_str::<T>(&res)?)

        // let res = client
        //     .get(format!("{}{}", self.base_config.rest_domain, get_url_params))
        //     .headers(headers)
        //     .send()
        //     .await?
        //     .json::<T>()
        //     .await?;

        // if self.debug {
        //     println!("[*] Response {:#?}", res);
        // }

        // Ok(res)
    }

    pub async fn post<T>(
        &self,
        request_path: &str,
        parameters: impl Serialize+ Debug,
    ) -> Result<T>
    where
        T: DeserializeOwned + std::fmt::Debug,
    {
        // # 获取本地时间
        let timestamp = self.get_timestamp();

        let data = serde_json::to_string(&parameters).unwrap();

        let message = format!("{}POST{}{}", timestamp, request_path, &data);

        let sign = self.sign(&message);

        let headers = self.create_header(&sign, &timestamp);

        let client = reqwest::Client::new();

        debug!("[*] Debug:parameters {:?}", parameters);

        debug!(
                "[*] Debug:\nUrl:{}\nparameters:{:?}",
                format!("{}{}", self.base_config.rest_domain, request_path),
                parameters
        );

        // let res = client
        //     .post(format!("{}{}", self.base_config.rest_domain, request_path))
        //     .headers(headers)
        //     .json(&parameters)
        //     .send()
        //     .await?
        //     .json::<T>()
        //     .await?;

        // if self.debug {
        //     println!("[*]Response {:#?}", res);
        // }

        // Ok(res)

        let res = client
            .post(format!("{}{}", self.base_config.rest_domain, request_path))
            .headers(headers)
            .json(&parameters)
            .send()
            .await?
            .text()
            .await?;

        debug!("[*] Response {:#?}", res);

        Ok(serde_json::from_str::<T>(&res)?)
    }

    pub async fn post_vec<T>(
        &self,
        request_path: &str,
        parameters: &Vec<BTreeMap<String, String>>,
    ) -> Result<T>
    where
        T: DeserializeOwned + std::fmt::Debug,
    {
        // # 获取本地时间
        let timestamp = self.get_timestamp();

        let data = serde_json::to_string(parameters).unwrap();

        let message = format!("{}POST{}{}", timestamp, request_path, &data);

        let sign = self.sign(&message);
        // println!("sign : {:?} ", sign);
        let headers = self.create_header(&sign, &timestamp);

        let client = reqwest::Client::new();

        debug!(
                "[*] Debug:\nUrl:{}\nparameters:{:?}",
                format!("{}{}", self.base_config.rest_domain, request_path),
                parameters
            );

        let res = client
            .post(format!("{}{}", self.base_config.rest_domain, request_path))
            .headers(headers)
            .json(&parameters)
            .send()
            .await?
            .text()
            .await?;

        // if self.debug {
        //     println!("[*] Response {:#?}", res);
        // }

        debug!("[*] Response {:#?}", res);

        Ok(serde_json::from_str::<T>(&res)?)
    }

    fn create_header(&self, sign: &str, timestamp: &str) -> HeaderMap {
        // 处理请求头 headers

        let mut header_map = HeaderMap::new();

        header_map.insert(
            "OK-ACCESS-KEY",
            HeaderValue::from_str(&self.api_key).unwrap(),
        );
        header_map.insert("OK-ACCESS-SIGN", HeaderValue::from_str(&sign).unwrap());
        header_map.insert(
            "OK-ACCESS-TIMESTAMP",
            HeaderValue::from_str(&timestamp).unwrap(),
        );
        header_map.insert(
            "OK-ACCESS-PASSPHRASE",
            HeaderValue::from_str(&self.passphrase).unwrap(),
        );
        header_map.insert(
            "CONTENT_TYPE",
            HeaderValue::from_static("application/json; charset=UTF-8"),
        );

        // 如果是测试网
        if self.base_config.testnet {
            header_map.insert("x-simulated-trading", HeaderValue::from_static("1"));
        }

        header_map
    }

    fn parse_params_to_str(&self, parameters: &BTreeMap<String, String>) -> String {
        parameters
            .into_iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<String>>()
            .join("&")
    }
    // 做签名
    fn sign(&self, message: &String) -> String {
        let hmac_key = ring::hmac::Key::new(hmac::HMAC_SHA256, &self.secret_key.as_bytes());
        let result = ring::hmac::sign(&hmac_key, &message.as_bytes());
        base64::encode(result)
    }

    pub fn get_timestamp(&self) -> String {
        chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string()
    }
}

impl OkxPublicClient {
    pub async fn get<T>(
        &self,
        request_path: &str,
        parameters: &BTreeMap<String, String>,
    ) -> Result<T>
        where
            T: DeserializeOwned + std::fmt::Debug,
    {
        // # 获取本地时间
        let timestamp = self.get_timestamp();

        let mut get_url_params: String = String::from(request_path);

        // 有数据就格式化参数
        if parameters.len() != 0 {
            get_url_params = format!("{}?{}", request_path, self.parse_params_to_str(parameters));
        }
        // println!("url {}",get_url_params);
        // OK-ACCESS-SIGN的请求头是对timestamp + method + requestPath + body字符串(+表示字符串连接)，以及SecretKey

        // let message = format!("{}GET{}{}", timestamp, get_url_params, &self.secret_key);
        let message = format!("{}GET{}", timestamp, get_url_params);

        // println!("{:?}", message);

        let headers = self.create_header(&timestamp);

        let client = reqwest::Client::new();

        debug!(
            "[*] Debug:\nUrl:{}\nparameters:{:?}",
            format!("{}{}", self.base_config.rest_domain, request_path),
            parameters
        );

        let res = client
            .get(format!("{}{}", self.base_config.rest_domain, get_url_params))
            .headers(headers)
            .send()
            .await?
            .text()
            .await?;

        debug!("[*] Response {:#?}", res);

        Ok(serde_json::from_str::<T>(&res)?)
    }

    pub async fn post<T>(
        &self,
        request_path: &str,
        parameters: impl Serialize+ Debug,
    ) -> Result<T>
        where
            T: DeserializeOwned + std::fmt::Debug,
    {
        // # 获取本地时间
        let timestamp = self.get_timestamp();

        let data = serde_json::to_string(&parameters).unwrap();

        let message = format!("{}POST{}{}", timestamp, request_path, &data);

        let headers = self.create_header( &timestamp);

        let client = reqwest::Client::new();

        debug!("[*] Debug:parameters {:?}", parameters);

        debug!(
                "[*] Debug:\nUrl:{}\nparameters:{:?}",
                format!("{}{}", self.base_config.rest_domain, request_path),
                parameters
        );

        let res = client
            .post(format!("{}{}", self.base_config.rest_domain, request_path))
            .headers(headers)
            .json(&parameters)
            .send()
            .await?
            .text()
            .await?;

        debug!("[*] Response {:#?}", res);

        Ok(serde_json::from_str::<T>(&res)?)
    }

    pub async fn post_vec<T>(
        &self,
        request_path: &str,
        parameters: &Vec<BTreeMap<String, String>>,
    ) -> Result<T>
        where
            T: DeserializeOwned + std::fmt::Debug,
    {
        // # 获取本地时间
        let timestamp = self.get_timestamp();

        let data = serde_json::to_string(parameters).unwrap();

        let message = format!("{}POST{}{}", timestamp, request_path, &data);

        // println!("sign : {:?} ", sign);
        let headers = self.create_header( &timestamp);

        let client = reqwest::Client::new();

        debug!(
                "[*] Debug:\nUrl:{}\nparameters:{:?}",
                format!("{}{}", self.base_config.rest_domain, request_path),
                parameters
            );

        let res = client
            .post(format!("{}{}", self.base_config.rest_domain, request_path))
            .headers(headers)
            .json(&parameters)
            .send()
            .await?
            .text()
            .await?;

        // if self.debug {
        //     println!("[*] Response {:#?}", res);
        // }

        debug!("[*] Response {:#?}", res);

        Ok(serde_json::from_str::<T>(&res)?)
    }

    fn create_header(&self, timestamp: &str) -> HeaderMap {
        // 处理请求头 headers

        let mut header_map = HeaderMap::new();

        header_map.insert(
            "OK-ACCESS-TIMESTAMP",
            HeaderValue::from_str(&timestamp).unwrap(),
        );
        header_map.insert(
            "CONTENT_TYPE",
            HeaderValue::from_static("application/json; charset=UTF-8"),
        );

        // 如果是测试网
        if self.base_config.testnet {
            header_map.insert("x-simulated-trading", HeaderValue::from_static("1"));
        }

        header_map
    }

    fn parse_params_to_str(&self, parameters: &BTreeMap<String, String>) -> String {
        parameters
            .into_iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<String>>()
            .join("&")
    }

    pub fn get_timestamp(&self) -> String {
        chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string()
    }
}