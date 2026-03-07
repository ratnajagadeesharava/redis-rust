use std::{collections::{HashMap, VecDeque}, net::TcpStream, time::SystemTime};

use crate::{client::ClientId, redisObject::RedisObject};

pub struct RedisDb {
    pub map: HashMap<String, RedisObject>,
    pub expiry_map: HashMap<String, SystemTime>,
    pub blocked:HashMap<String,VecDeque<ClientId>>
}

impl RedisDb {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            expiry_map: HashMap::new(),
            blocked:HashMap::new()
        }
    }
}
