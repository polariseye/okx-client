use std::collections::HashMap;

pub struct WeightLimit {
    total_weight: u64,
    refresh_seconds: u32,

    last_refresh_time: u64,
    used_weight: u64
}

impl WeightLimit {
    pub fn new(total_weight: u64, refresh_seconds: u32) -> Self {
        Self {
            total_weight,
            refresh_seconds,
            last_refresh_time: 0,
            used_weight: 0,
        }
    }

    pub fn request_once(&mut self, weight: u64) -> bool{
        let now = (super::get_unix()/1000) as u64;
        if now > self.refresh_seconds as u64 + self.last_refresh_time {
            self.used_weight = 0;
            self.last_refresh_time = now;
        }

        if self.used_weight + weight > self.total_weight {
            return false;
        }

        self.used_weight = self.used_weight + weight;
        true
    }

    pub fn check_valid(&mut self, weight: u64) -> bool{
        let now = (super::get_unix()/1000) as u64;
        if now > self.refresh_seconds as u64 + self.last_refresh_time {
            self.used_weight = 0;
            self.last_refresh_time = now;
        }

        if self.used_weight + weight > self.total_weight {
            return false;
        }

        true
    }
}

pub struct LimitMgr {
    data: HashMap<u32, Vec<WeightLimit>>
}

impl LimitMgr {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn add_limit(&mut self, api_id: u32, weights: Vec<WeightLimit>) {
        self.data.insert(api_id, weights);
    }

    pub fn request_once(&mut self, api_id: u32) -> bool {
        match self.data.get_mut(&api_id) {
            None => {
                panic!("api not register:{}", api_id);
            }
            Some(val) => {
                for item in val.iter_mut() {
                    if !item.check_valid(1) {
                        return false;
                    }
                }
                for item in val.iter_mut() {
                    item.request_once(1);
                }

                return true;
            }
        }
    }
}