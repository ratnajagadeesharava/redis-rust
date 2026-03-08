use std::{
    collections::{HashMap, VecDeque},
    net::TcpStream,
    time::SystemTime,
};

use crate::{client::ClientId, redisObject::RedisObject};

pub struct RedisDb {
    pub map: HashMap<String, RedisObject>,
    pub expiry_map: HashMap<String, SystemTime>,
    pub blocked: HashMap<String, VecDeque<ClientId>>,
    pub last_id: u128,
    pub last_sequence_number: u32,
}

impl RedisDb {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            expiry_map: HashMap::new(),
            blocked: HashMap::new(),
            last_id: 0,
            last_sequence_number: 0,
        }
    }
}
