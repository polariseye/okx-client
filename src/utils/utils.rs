use chrono::format::DelayedFormat;
use chrono::format::StrftimeItems;
use chrono::prelude::*;
use serde::{de, Deserialize, Deserializer, Serializer};
use std::fmt::Display;
use std::str::FromStr;

pub async fn get_send_time() -> String {
    let fmt = "%Y-%m-%d %H:%M:%S";

    let now: DateTime<Local> = Local::now();

    let dft: DelayedFormat<StrftimeItems> = now.format(fmt);
    let str_date: String = dft.to_string();
    // 2021-01-04 20:02:09
    str_date
}

// 单位: 毫秒
pub fn get_unix() -> i64 {
    (get_unix_nano() / 1000000i128) as i64
}

pub fn get_unix_nano() -> i128 {
    time::OffsetDateTime::now_utc().unix_timestamp_nanos()
}

pub fn to_str<T, S>(val: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
{
    serializer.serialize_str(&val.to_string())
}

pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr + Default,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(Default::default());
    }

    T::from_str(&s).map_err(de::Error::custom)
}

pub fn from_opt_str<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }

    Ok(Some(T::from_str(&s).map_err(de::Error::custom)?))
}

pub fn to_opt_str<T, S>(val: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
{
    match val {
        Some(val) => {
            serializer.serialize_str(&val.to_string())
        },
        None => {
            serializer.serialize_none()
        }
    }

}