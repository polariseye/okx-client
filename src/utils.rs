use chrono::format::DelayedFormat;
use chrono::format::StrftimeItems;
use chrono::prelude::*;
use serde::{de, Deserialize, Deserializer};
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

pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
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

/// Serialises number as a string; deserialises either as a string or number.
///
/// This format works for `u64`, `u128`, `Option<u64>` and `Option<u128>` types.
/// When serialising, numbers are serialised as decimal strings.  When
/// deserialising, strings are parsed as decimal numbers while numbers are
/// interpreted as is.
pub mod dec_format {
    use serde::de;
    use serde::{Deserializer, Serializer};

    /// Abstraction between integers that we serialise.
    pub trait DecType: Sized {
        /// Formats number as a decimal string; passes `None` as is.
        fn serialize(&self) -> Option<String>;

        /// Constructs Self from a `null` value.  Returns error if this type
        /// does not accept `null` values.
        fn try_from_unit() -> Result<Self, ()> {
            Err(())
        }

        /// Tries to parse decimal string as an integer.
        fn try_from_str(value: &str) -> Result<Self, std::num::ParseIntError>;

        /// Constructs Self from a 64-bit unsigned integer.
        fn from_u64(value: u64) -> Self;
    }

    impl DecType for u64 {
        fn serialize(&self) -> Option<String> {
            Some(self.to_string())
        }
        fn try_from_str(value: &str) -> Result<Self, std::num::ParseIntError> {
            Self::from_str_radix(value, 10)
        }
        fn from_u64(value: u64) -> Self {
            value
        }
    }

    impl DecType for u128 {
        fn serialize(&self) -> Option<String> {
            Some(self.to_string())
        }
        fn try_from_str(value: &str) -> Result<Self, std::num::ParseIntError> {
            Self::from_str_radix(value, 10)
        }
        fn from_u64(value: u64) -> Self {
            value.into()
        }
    }

    impl<T: DecType> DecType for Option<T> {
        fn serialize(&self) -> Option<String> {
            self.as_ref().and_then(DecType::serialize)
        }
        fn try_from_unit() -> Result<Self, ()> {
            Ok(None)
        }
        fn try_from_str(value: &str) -> Result<Self, std::num::ParseIntError> {
            Some(T::try_from_str(value)).transpose()
        }
        fn from_u64(value: u64) -> Self {
            Some(T::from_u64(value))
        }
    }

    struct Visitor<T>(core::marker::PhantomData<T>);

    impl<'de, T: DecType> de::Visitor<'de> for Visitor<T> {
        type Value = T;

        fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            fmt.write_str("a non-negative integer as a string")
        }

        fn visit_unit<E: de::Error>(self) -> Result<T, E> {
            T::try_from_unit().map_err(|()| de::Error::invalid_type(de::Unexpected::Option, &self))
        }

        fn visit_u64<E: de::Error>(self, value: u64) -> Result<T, E> {
            Ok(T::from_u64(value))
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<T, E> {
            T::try_from_str(value).map_err(de::Error::custom)
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
        where
            D: Deserializer<'de>,
            T: DecType,
    {
        deserializer.deserialize_any(Visitor(Default::default()))
    }

    pub fn serialize<S, T>(num: &T, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
            T: DecType,
    {
        match num.serialize() {
            Some(value) => serializer.serialize_str(&value),
            None => serializer.serialize_none(),
        }
    }
}