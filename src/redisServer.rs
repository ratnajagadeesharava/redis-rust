use std::{
    collections::{HashMap, VecDeque, btree_map::Keys},
    io::{Read, Write},
    net::TcpStream,
    str::from_utf8,
    time::{Duration, SystemTime},
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
            RedisCommand::Unkown => todo!(),
            RedisCommand::Ping => self.ping(clientId),
            RedisCommand::LRANGE(key, start, end) => self.lrange(clientId, key, start, end),
            RedisCommand::LPush(key, value) => self.l_push(clientId, key, value),
            RedisCommand::LLEN(key) => self.list_length(clientId, key),
            RedisCommand::LPOP(key, count) => self.left_pop(clientId, key, count),
            RedisCommand::BLPOP(key, timeout) => self.blocked_pop(clientId, key, timeout),
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
                            client.blocked = true;
                            client.waiting_key = Some(key);
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
            client.blocked = true;
            client.waiting_key = Some(key);
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
                println!("{:?}", command_array);
                let redisCommnad = array_to_command(&command_array);
                self.execute(redisCommnad, clientId);
            }
        }
    }
    fn check_blocked(&mut self,key: &String,values:Vec<String>)->bool{
       if self.redis_db.blocked.contains_key(key){
        return match self.redis_db.blocked.get_mut(key).unwrap().pop_front(){
            Some(clientId) =>{
                let client =self.client_map.get_mut(&clientId).unwrap();
                client.blocked = false;
                client.waiting_key = None;
                client.stream.borrow_mut().write_all(&parse_resp(Resp::BulkString(values[0].clone()))).unwrap();
                true
            },
            None => false,
        }
        
       }{
        return false;
       }
    }
    pub fn l_push(&mut self, clientId: ClientId, key: String, values: Vec<String>) {
       if self.check_blocked(&key, values){
        return ;
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
        if self.check_blocked(&key, values){
        return ;
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
