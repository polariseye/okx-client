use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use crate::{InstType, OkxError};

#[derive(Debug)]
pub struct WeightLimit {
    total_weight: u32,
    refresh_seconds: u32,

    last_refresh_time: u32,
    used_weight: u32
}

impl WeightLimit {
    pub fn new(total_weight: u32, refresh_seconds: u32) -> Self {
        Self {
            total_weight,
            refresh_seconds,
            last_refresh_time: 0,
            used_weight: 0,
        }
    }

    pub fn request_once(&mut self, weight: u32) -> bool{
        let now = (super::get_unix()/1000) as u32;
        if now > self.refresh_seconds as u32 + self.last_refresh_time {
            self.used_weight = 0;
            self.last_refresh_time = now;
        }

        if self.used_weight + weight > self.total_weight {
            return false;
        }

        self.used_weight = self.used_weight + weight;
        true
    }

    pub fn check_valid(&mut self, weight: u32) -> bool{
        let now = (super::get_unix()/1000) as u32;
        if now > self.refresh_seconds as u32 + self.last_refresh_time {
            self.used_weight = 0;
            self.last_refresh_time = now;
        }

        if self.used_weight + weight > self.total_weight {
            return false;
        }

        true
    }
}

#[derive(Debug)]
pub struct LimitMgr {
    data: RwLock<HashMap<u32, Arc<Mutex<WeightLimit>>>>,
    data_with_inst_id: RwLock<HashMap<u32, HashMap<String, Arc<Mutex<WeightLimit>>>>>,
    data_with_inst_type: RwLock<HashMap<u32, HashMap<InstType, Arc<Mutex<WeightLimit>>>>>,
    data_with_inst_family: RwLock<HashMap<u32, HashMap<String, Arc<Mutex<WeightLimit>>>>>
}

impl LimitMgr {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            data_with_inst_type: RwLock::new(HashMap::new()),
            data_with_inst_id: RwLock::new(HashMap::new()),
            data_with_inst_family: RwLock::new(HashMap::new()),
        }
    }

    fn get_or_add_limit(&self, api_id: u32, total_weight: u32, refresh_seconds: u32) -> Arc<Mutex<WeightLimit>>{
        {
            let reader = self.data.read().unwrap();
            if let Some(val) = reader.get(&api_id) {
                return val.clone();
            }
        }

        let mut writer = self.data.write().unwrap();
        if let Some(val) = writer.get(&api_id) {
            return val.clone();
        }

        let result = Arc::new(Mutex::new(WeightLimit::new(total_weight, refresh_seconds)));
        writer.insert(api_id,  result.clone());

        result
    }

    pub fn check_limit(& self, api_id: u32, weight: u32, total_weight:u32, refresh_seconds: u32) -> Result<(), OkxError> {
        let limit_obj = self.get_or_add_limit(api_id, total_weight, refresh_seconds);
        let mut writer = limit_obj.lock().unwrap();
        if !writer.request_once(weight) {
            Err(OkxError::RateLimit)
        } else {
            Ok(())
        }
    }

    fn get_or_add_inst_id_limit(&self, api_id: u32, inst_id: &str, total_weight: u32, refresh_seconds: u32) -> Arc<Mutex<WeightLimit>>{
        {
            let reader = self.data_with_inst_id.read().unwrap();
            if let Some(val) = reader.get(&api_id) {
                if let Some(val) = val.get(inst_id) {
                    return val.clone();
                }
            }
        }

        let mut writer = self.data_with_inst_id.write().unwrap();
        if let Some(val) = writer.get_mut(&api_id) {
            if let Some(val) = val.get_mut(inst_id) {
                val.clone()
            } else {
                let result = Arc::new(Mutex::new(WeightLimit::new(total_weight, refresh_seconds)));
                val.insert(inst_id.to_string(), result.clone());

                result
            }
        } else {
            let result = Arc::new(Mutex::new(WeightLimit::new(total_weight, refresh_seconds)));
            let mut data =HashMap::new();
            data.insert(inst_id.to_string(), result.clone());
            writer.insert(api_id,  data);

            result
        }
    }

    pub fn check_limit_with_inst_id(& self, api_id: u32, inst_id: &str, weight: u32, total_weight:u32, refresh_seconds: u32) -> Result<(), OkxError> {
        let limit_obj = self.get_or_add_inst_id_limit(api_id, inst_id, total_weight, refresh_seconds);
        let mut writer = limit_obj.lock().unwrap();
        if !writer.request_once(weight) {
            Err(OkxError::RateLimit)
        } else {
            Ok(())
        }
    }

    fn get_or_add_inst_type_limit(&self, api_id: u32, inst_type: InstType, total_weight: u32, refresh_seconds: u32) -> Arc<Mutex<WeightLimit>>{
        {
            let reader = self.data_with_inst_type.read().unwrap();
            if let Some(val) = reader.get(&api_id) {
                if let Some(val) = val.get(&inst_type) {
                    return val.clone();
                }
            }
        }

        let mut writer = self.data_with_inst_type.write().unwrap();
        if let Some(val) = writer.get_mut(&api_id) {
            if let Some(val) = val.get_mut(&inst_type) {
                val.clone()
            } else {
                let result = Arc::new(Mutex::new(WeightLimit::new(total_weight, refresh_seconds)));
                val.insert(inst_type, result.clone());

                result
            }
        } else {
            let result = Arc::new(Mutex::new(WeightLimit::new(total_weight, refresh_seconds)));
            let mut data =HashMap::new();
            data.insert(inst_type, result.clone());
            writer.insert(api_id,  data);

            result
        }
    }

    pub fn check_limit_with_inst_type(& self, api_id: u32, inst_type: InstType, weight: u32, total_weight:u32, refresh_seconds: u32) -> Result<(), OkxError> {
        let limit_obj = self.get_or_add_inst_type_limit(api_id, inst_type, total_weight, refresh_seconds);
        let mut writer = limit_obj.lock().unwrap();
        if !writer.request_once(weight) {
            Err(OkxError::RateLimit)
        } else {
            Ok(())
        }
    }

    fn get_or_add_inst_family_limit(&self, api_id: u32, inst_family: &str, total_weight: u32, refresh_seconds: u32) -> Arc<Mutex<WeightLimit>>{
        {
            let reader = self.data_with_inst_family.read().unwrap();
            if let Some(val) = reader.get(&api_id) {
                if let Some(val) = val.get(inst_family) {
                    return val.clone();
                }
            }
        }

        let mut writer = self.data_with_inst_family.write().unwrap();
        if let Some(val) = writer.get_mut(&api_id) {
            if let Some(val) = val.get_mut(inst_family) {
                val.clone()
            } else {
                let result = Arc::new(Mutex::new(WeightLimit::new(total_weight, refresh_seconds)));
                val.insert(inst_family.to_string(), result.clone());

                result
            }
        } else {
            let result = Arc::new(Mutex::new(WeightLimit::new(total_weight, refresh_seconds)));
            let mut data =HashMap::new();
            data.insert(inst_family.to_string(), result.clone());
            writer.insert(api_id,  data);

            result
        }
    }

    pub fn check_limit_with_inst_family(&self, api_id: u32, inst_family: &str, weight: u32, total_weight:u32, refresh_seconds: u32) -> Result<(), OkxError> {
        let limit_obj = self.get_or_add_inst_family_limit(api_id, inst_family, total_weight, refresh_seconds);
        let mut writer = limit_obj.lock().unwrap();
        if !writer.request_once(weight) {
            Err(OkxError::RateLimit)
        } else {
            Ok(())
        }
    }
}