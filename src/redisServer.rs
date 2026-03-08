use std::{
    collections::{HashMap, VecDeque, btree_map::Keys},
    io::{Read, Write},
    net::TcpStream,
    str::from_utf8,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{
    client::{self, Client, ClientId},
    list::List,
    redisCommand::{RedisCommand, array_to_command},
    redisDb::RedisDb,
    redisObject::{DataType, RedisObject},
    resp::{Resp, parse_message, parse_resp},
};

pub struct RedisServer {
    pub client_map: HashMap<usize, Client>,
    pub redis_db: RedisDb,
}

impl RedisServer {
    pub fn execute(&mut self, cmd: RedisCommand, clientId: ClientId) {
        match cmd {
            RedisCommand::Set(key, value, ttl) => self.set_command(clientId, key, value, ttl),
            RedisCommand::Get(key) => self.get_command(clientId, key),
            RedisCommand::RPush(key, value) => self.r_push(clientId, key, value),
            RedisCommand::Echo(value) => self.echo(clientId, value),

            RedisCommand::Ping => self.ping(clientId),
            RedisCommand::LRANGE(key, start, end) => self.lrange(clientId, key, start, end),
            RedisCommand::LPush(key, value) => self.l_push(clientId, key, value),
            RedisCommand::LLEN(key) => self.list_length(clientId, key),
            RedisCommand::LPOP(key, count) => self.left_pop(clientId, key, count),
            RedisCommand::BLPOP(key, timeout) => self.blocked_pop(clientId, key, timeout),
            RedisCommand::TYPE(key) => self.find_type(clientId, key),
            RedisCommand::XADD(key, id, key_values) => self.xadd(clientId, &key, &id, key_values),
            RedisCommand::Unkown => todo!(),
        }
    }

    fn xadd(
        &mut self,
        clientId: ClientId,
        key: &String,
        id: &String,
        key_values: Vec<(String, String)>,
    ) {
        // let now = SystemTime::now();
        // let duration_since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
        // let duration_in_milli_seconds = duration_since_epoch.as_millis().to_string();
        let client = self.client_map.get(&clientId).unwrap();
        let id_split_vec: Vec<&str> = id.split("-").collect();
        let id_u64: u128 = id_split_vec[0].parse().unwrap();
        let sequence_number: u32 = id_split_vec[1].parse().unwrap();
        let exisiting_id = self.redis_db.last_id;
        let exisiting_sq_no = self.redis_db.last_sequence_number;
        if id_u64 == self.redis_db.last_id {
            if sequence_number <= self.redis_db.last_sequence_number {
                self.write_to_client(clientId, Resp::Error(format!("The ID specified in XADD is equal or smaller than the target stream top item")));
                return;
            } else {
                self.redis_db.last_sequence_number = sequence_number;
            }
        } else if id_u64 < self.redis_db.last_id {
            self.write_to_client(
                clientId,
                Resp::Error(
                    format!("The ID specified in XADD is equal or smaller than {exisiting_id}-{exisiting_sq_no}")
                ),
            );
            return;
        } else {
            self.redis_db.last_id = id_u64;
            self.redis_db.last_sequence_number = sequence_number;
        }
        //
        if self.redis_db.map.contains_key(key) {
            if let Some(obj) = self.redis_db.map.get_mut(key) {
                if let DataType::STREAM(map) = &mut obj.data {
                    if let Some(key_value_map) = map.get_mut(id) {
                        for val in key_values {
                            key_value_map.insert(val.0, val.1);
                        }
                    }
                }
            }
        } else {
            let mut obj = RedisObject {
                data: DataType::STREAM(HashMap::new()),
            };
            if let DataType::STREAM(map) = &mut obj.data {
                map.insert(id.clone(), HashMap::new());
                if let Some(key_value_map) = map.get_mut(id) {
                    for val in key_values {
                        key_value_map.insert(val.0, val.1);
                    }
                }
            }
            self.redis_db.map.insert(key.clone(), obj);
        }
        self.write_to_client(clientId, Resp::BulkString(id.clone()));
    }
    #[inline]
    fn write_to_client(&mut self, clientId: ClientId, val: Resp) {
        let client = self.client_map.get_mut(&clientId).unwrap();
        client
            .stream
            .borrow_mut()
            .write_all(&parse_resp(val))
            .unwrap();
    }

    fn find_type(&mut self, clientId: ClientId, key: String) {
        match self.redis_db.map.get_mut(&key) {
            Some(obj) => match &obj.data {
                DataType::STRING(_) => {
                    self.write_to_client(clientId, Resp::SimpleString("string".to_string()));
                }
                DataType::LIST(list) => {
                    self.write_to_client(clientId, Resp::SimpleString("list".to_string()));
                }
                DataType::STREAM(map) => {
                    self.write_to_client(clientId, Resp::SimpleString("stream".to_string()));
                }
            },
            None => {
                self.write_to_client(clientId, Resp::SimpleString("none".to_string()));
            }
        }
    }

    fn blocked_pop(&mut self, clientId: ClientId, key: String, timeout: i32) {
        let client = self.client_map.get_mut(&clientId).unwrap();
        if self.redis_db.map.contains_key(&key) {
            if let Some(obj) = self.redis_db.map.get_mut(&key) {
                if let DataType::LIST(list) = &mut obj.data {
                    match list.pop_front() {
                        Some(node) => {
                            let val = node.borrow_mut().val.clone();
                            client
                                .stream
                                .borrow_mut()
                                .write_all(&parse_resp(Resp::BulkString(val)))
                                .unwrap();
                        }
                        None => {
                            self.redis_db
                                .blocked
                                .entry(key.clone())
                                .or_insert(VecDeque::new())
                                .push_back(clientId);
                            println!("blocked {clientId}");
                            client.blocked = true;
                            client.waiting_key = Some(key);
                            client.waiting_time = if timeout == 0 {
                                None
                            } else {
                                Some(SystemTime::now() + Duration::from_millis(timeout as u64))
                            };
                        }
                    }
                }
            }
        } else {
            self.redis_db
                .blocked
                .entry(key.clone())
                .or_insert(VecDeque::new())
                .push_back(clientId);
            println!("blocked {clientId}");
            client.blocked = true;
            client.waiting_key = Some(key);
            client.waiting_time = if timeout == 0 {
                None
            } else {
                Some(SystemTime::now() + Duration::from_millis(timeout as u64))
            };
        }
    }

    fn left_pop(&mut self, clientId: ClientId, key: String, count: i32) {
        let mut client = self.client_map.get(&clientId).unwrap();
        if self.redis_db.map.contains_key(&key) {
            if let Some(obj) = self.redis_db.map.get_mut(&key) {
                if let DataType::LIST(list) = &mut obj.data {
                    let mut popped_items = Vec::<String>::new();
                    let mut count = count;
                    while count != 0 {
                        match list.pop_front() {
                            Some(node) => {
                                let val = node.borrow_mut().val.clone();
                                popped_items.push(val);
                            }
                            None => {
                                // client.stream.write(b"$-1\r\n").unwrap();
                                break;
                            }
                        }
                        count -= 1;
                    }
                    if popped_items.len() > 1 {
                        client
                            .stream
                            .borrow_mut()
                            .write_all(&parse_resp(Resp::Array(popped_items)))
                            .unwrap()
                    } else {
                        if popped_items.len() == 1 {
                            client
                                .stream
                                .borrow_mut()
                                .write_all(&parse_resp(Resp::BulkString(popped_items[0].clone())))
                                .unwrap();
                        } else {
                            client.stream.borrow_mut().write_all(b"$-1\r\n").unwrap();
                        }
                    }
                }
            } else {
                client.stream.borrow_mut().write_all(b"$-1\r\n").unwrap();
            }
        }
    }

    fn list_length(&mut self, clientId: ClientId, key: String) {
        let client = self.client_map.get(&clientId).unwrap();
        if self.redis_db.map.contains_key(&key) {
            if let Some(obj) = self.redis_db.map.get_mut(&key) {
                if let DataType::LIST(list) = &mut obj.data {
                    let count = list.count;

                    client
                        .stream
                        .borrow_mut()
                        .write_all(&parse_resp(Resp::Integer(count)))
                        .unwrap();
                }
            }
        } else {
            client
                .stream
                .borrow_mut()
                .write_all(&parse_resp(Resp::Integer(0)))
                .unwrap()
        }
    }

    fn lrange(&mut self, clientId: ClientId, key: String, start: i32, end: i32) {
        let client = self.client_map.get(&clientId).unwrap();
        if self.redis_db.map.contains_key(&key) {
            if let Some(obj) = self.redis_db.map.get_mut(&key) {
                if let DataType::LIST(list) = &mut obj.data {
                    let count = list.count;
                    let mut s = start;
                    let mut e = end;
                    if start < 0 {
                        s = (count as i32 + start).max(0);
                    }
                    if end < 0 {
                        e = count as i32 + end;
                    }
                    println!("{} -- > {}  --- {}", s, e, count);
                    let values = list.range(s as usize, e as usize);
                    println!("{:?}", values);
                    client
                        .stream
                        .borrow_mut()
                        .write_all(&parse_resp(Resp::Array(values)))
                        .unwrap();
                }
            }
        } else {
            client
                .stream
                .borrow_mut()
                .write_all(&parse_resp(Resp::Array(vec![])))
                .unwrap()
        }
    }

    fn set_command(&mut self, clientId: ClientId, key: String, value: String, ttl: Option<u64>) {
        let client = self.client_map.get(&clientId).unwrap();
        let obj = RedisObject {
            data: DataType::STRING(value),
        };
        self.redis_db.map.insert(key.clone(), obj);
        match ttl {
            Some(val) => {
                let expiry_time = SystemTime::now() + Duration::from_millis(val);
                self.redis_db.expiry_map.insert(key.clone(), expiry_time);
            }
            None => {}
        }
        client
            .stream
            .borrow_mut()
            .write_all(&parse_resp(Resp::SimpleString(String::from("OK"))))
            .unwrap()
    }

    fn get_command(&mut self, clientId: ClientId, key: String) {
        let client = self.client_map.get(&clientId).unwrap();
        if self.redis_db.map.contains_key(&key) {
            if let Some(exp_time) = self.redis_db.expiry_map.get(&key) {
                if *exp_time < SystemTime::now() {
                    self.redis_db.expiry_map.remove(&key);
                    self.redis_db.map.remove(&key);
                    client.stream.borrow_mut().write_all(b"$-1\r\n").unwrap();
                }
            }

            if let Some(obj) = self.redis_db.map.get(&key) {
                if let DataType::STRING(val) = &obj.data {
                    println!("pear {val}");
                    client
                        .stream
                        .borrow_mut()
                        .write_all(&parse_resp(Resp::BulkString(val.clone())))
                        .unwrap();
                }
            }
        }
    }

    fn echo(&mut self, clientId: ClientId, value: String) {
        let client = self.client_map.get(&clientId).unwrap();
        client
            .stream
            .borrow_mut()
            .write_all(&parse_resp(Resp::BulkString(value)))
            .unwrap();
    }

    fn ping(&mut self, clientId: ClientId) {
        let client = self.client_map.get(&clientId).unwrap();
        client
            .stream
            .borrow_mut()
            .write_all(&parse_resp(Resp::SimpleString(String::from("PONG"))));
    }

    pub fn execute_stream(&mut self, clientId: ClientId) {
        let mut buffer = [0; 1024];
        let client = self.client_map.get(&clientId).unwrap();
        let bytes_read = match client.stream.borrow_mut().read(&mut buffer) {
            Ok(bytes_read) => bytes_read,
            Err(_) => 0,
        };
        if bytes_read != 0 {
            let message = from_utf8(&buffer[..bytes_read]).unwrap();
            // println!("{:?}",message);
            if let Resp::Array(arr) = parse_message(message) {
                let array_iter = arr.into_iter();
                println!("{:?}", message);
                let command_array: Vec<String> =
                    array_iter.filter(|val: &String| val.len() != 0).collect();
                println!("actual commands {:?}", command_array);
                let redisCommnad = array_to_command(&command_array);
                self.execute(redisCommnad, clientId);
            }
        }
    }
    fn check_blocked(
        &mut self,
        key: &String,
        values: &Vec<String>,
        current_client_id: ClientId,
    ) -> bool {
        if self.redis_db.blocked.contains_key(key) {
            return match self.redis_db.blocked.get_mut(key).unwrap().pop_front() {
                Some(clientId) => {
                    let client = self.client_map.get_mut(&clientId).unwrap();
                    client.blocked = false;
                    client.waiting_key = None;
                    client.waiting_time = None;
                    let mut result = Vec::<String>::new();
                    result.push(key.clone());
                    result.push(values[0].clone());
                    client
                        .stream
                        .borrow_mut()
                        .write_all(&parse_resp(Resp::Array(result)))
                        .unwrap();
                    println!("unblocked ting tong");
                    let current_client = self.client_map.get_mut(&current_client_id).unwrap();
                    current_client
                        .stream
                        .borrow_mut()
                        .write_all(&parse_resp(Resp::Integer(1)))
                        .unwrap();
                    true
                }
                None => false,
            };
        }
        {
            return false;
        }
    }
    pub fn l_push(&mut self, clientId: ClientId, key: String, values: Vec<String>) {
        if self.check_blocked(&key, &values, clientId) {
            return;
        }
        let client = self.client_map.get(&clientId).unwrap();
        if self.redis_db.map.contains_key(&key) {
            if let Some(obj) = self.redis_db.map.get_mut(&key) {
                if let DataType::LIST(list) = &mut obj.data {
                    println!("order given{:?}", values);
                    for value in values {
                        list.push_front(value);
                    }
                    client
                        .stream
                        .borrow_mut()
                        .write_all(&parse_resp(Resp::Integer(list.count)))
                        .unwrap()
                }
            }
        } else {
            let mut list = List::new();
            let mut count = 0;

            for value in values {
                list.push_front(value);
                count += 1;
            }
            let obj = RedisObject {
                data: DataType::LIST(list),
            };
            self.redis_db.map.insert(key, obj);
            client
                .stream
                .borrow_mut()
                .write_all(&parse_resp(Resp::Integer(count)))
                .unwrap()
        }
    }

    pub fn r_push(&mut self, clientId: ClientId, key: String, values: Vec<String>) {
        if self.check_blocked(&key, &values, clientId) {
            return;
        }
        println!("rpush");
        let client = self.client_map.get(&clientId).unwrap();
        if self.redis_db.map.contains_key(&key) {
            if let Some(obj) = self.redis_db.map.get_mut(&key) {
                if let DataType::LIST(list) = &mut obj.data {
                    for value in values {
                        list.push_back(value);
                    }
                    client
                        .stream
                        .borrow_mut()
                        .write_all(&parse_resp(Resp::Integer(list.count)))
                        .unwrap()
                }
            }
        } else {
            let mut list = List::new();
            let mut count = 0;
            for value in values {
                list.push_back(value);
                count += 1;
            }
            let obj = RedisObject {
                data: DataType::LIST(list),
            };
            self.redis_db.map.insert(key, obj);
            client
                .stream
                .borrow_mut()
                .write_all(&parse_resp(Resp::Integer(count)))
                .unwrap()
        }
    }
}
