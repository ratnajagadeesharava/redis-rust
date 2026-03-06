use std::{collections::HashMap, time::SystemTime};

use crate::redisObject::RedisObject;

pub struct RedisDb {
    pub map: HashMap<String, RedisObject>,
    pub expiry_map: HashMap<String, SystemTime>,
}

impl RedisDb {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            expiry_map: HashMap::new(),
        }
    }
}
