use serde::de;
use serde::{Deserialize, Deserializer};

pub fn default_perent() -> f32 {
    0.0
}

pub fn de_float_from_str<'a, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'a>,
{
    let str_val = String::deserialize(deserializer)?;

    if str_val == "".to_string() {
        return Ok(0.0);
    }
    str_val.parse::<f32>().map_err(de::Error::custom)
}
