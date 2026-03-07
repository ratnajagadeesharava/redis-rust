use std::{
    collections::btree_map::Keys,
    io::{Read, Write},
    net::TcpStream,
    os::unix::raw::pid_t,
    str::from_utf8,
    time::{Duration, SystemTime},
};

use crate::{
    list::List,
    redisCommand::{RedisCommand, array_to_command},
    redisDb::RedisDb,
    redisObject::{DataType, RedisObject},
    resp::{Resp, parse_message, parse_resp},
};

pub struct RedisServer;
impl RedisServer {
    pub fn execute(cmd: RedisCommand, redisDb: &mut RedisDb, stream: &mut TcpStream) {
        match cmd {
            RedisCommand::Set(key, value, ttl) => {
                Self::set_command(stream, redisDb, key, value, ttl)
            }
            RedisCommand::Get(key) => Self::get_command(stream, redisDb, key),
            RedisCommand::RPush(key, value) => Self::r_push(stream, redisDb, key, value),
            RedisCommand::Echo(value) => Self::echo(stream, value),
            RedisCommand::Unkown => todo!(),
            RedisCommand::Ping => Self::ping(stream),
            RedisCommand::LRANGE(key, start, end) => Self::lrange(stream, redisDb, key, start, end),
            RedisCommand::LPush(key, value) => Self::l_push(stream, redisDb, key, value),
            RedisCommand::LLEN(key) => Self::list_length(stream, redisDb, key),
            RedisCommand::LPOP(key, count) => Self::left_pop(stream, redisDb, key, count),
        }
    }
    fn left_pop(stream: &mut TcpStream, redisDb: &mut RedisDb, key: String, count: i32) {
        if redisDb.map.contains_key(&key) {
            if let Some(obj) = redisDb.map.get_mut(&key) {
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
                                // stream.write(b"$-1\r\n").unwrap();
                                break;
                            }
                        }
                        count -= 1;
                    }
                    if popped_items.len() > 1 {
                        stream
                            .write_all(&parse_resp(Resp::Array(popped_items)))
                            .unwrap()
                    } else {
                        if popped_items.len()==1{
                        stream.write_all(&parse_resp(Resp::BulkString(popped_items[0].clone()))).unwrap();
                        }else{
                            stream.write(b"$-1\r\n").unwrap();
                        }
                    }
            }
        } else {
            stream.write(b"$-1\r\n").unwrap();
        }
    }
    fn list_length(stream: &mut TcpStream, redisDb: &mut RedisDb, key: String) {
        if redisDb.map.contains_key(&key) {
            if let Some(obj) = redisDb.map.get_mut(&key) {
                if let DataType::LIST(list) = &mut obj.data {
                    let count = list.count;

                    stream.write_all(&parse_resp(Resp::Integer(count))).unwrap();
                }
            }
        } else {
            stream.write_all(&parse_resp(Resp::Integer(0))).unwrap()
        }
    }

    fn lrange(stream: &mut TcpStream, redisDb: &mut RedisDb, key: String, start: i32, end: i32) {
        if redisDb.map.contains_key(&key) {
            if let Some(obj) = redisDb.map.get_mut(&key) {
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
                    stream.write_all(&parse_resp(Resp::Array(values))).unwrap();
                }
            }
        } else {
            stream.write_all(&parse_resp(Resp::Array(vec![]))).unwrap()
        }
    }
    fn set_command(
        stream: &mut TcpStream,
        redisDb: &mut RedisDb,
        key: String,
        value: String,
        ttl: Option<u64>,
    ) {
        let obj = RedisObject {
            data: DataType::STRING(value),
        };
        redisDb.map.insert(key.clone(), obj);
        match ttl {
            Some(val) => {
                let expiry_time = SystemTime::now() + Duration::from_millis(val);
                redisDb.expiry_map.insert(key.clone(), expiry_time);
            }
            None => {}
        }
        stream
            .write_all(&parse_resp(Resp::SimpleString(String::from("OK"))))
            .unwrap()
    }

    fn get_command(stream: &mut TcpStream, redisDb: &mut RedisDb, key: String) {
        if redisDb.map.contains_key(&key) {
            if let Some(exp_time) = redisDb.expiry_map.get(&key) {
                if *exp_time < SystemTime::now() {
                    redisDb.expiry_map.remove(&key);
                    redisDb.map.remove(&key);
                    stream.write(b"$-1\r\n").unwrap();
                }
            }

            if let Some(obj) = redisDb.map.get(&key) {
                if let DataType::STRING(val) = &obj.data {
                    println!("pear {val}");
                    stream
                        .write_all(&parse_resp(Resp::BulkString(val.clone())))
                        .unwrap();
                }
            }
        }
    }
    fn echo(stream: &mut TcpStream, value: String) {
        stream
            .write_all(&parse_resp(Resp::BulkString(value)))
            .unwrap();
    }
    fn ping(stream: &mut TcpStream) {
        stream.write_all(&parse_resp(Resp::SimpleString(String::from("PONG"))));
    }
    pub fn execute_stream(redisDb: &mut RedisDb, stream: &mut TcpStream) {
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
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

                        Self::execute(redisCommnad, redisDb, stream);
                    }
                }
            }
            Err(_) => {}
        }
    }
    pub fn l_push(stream: &mut TcpStream, redisDb: &mut RedisDb, key: String, values: Vec<String>) {
        println!("lpush");
        if redisDb.map.contains_key(&key) {
            if let Some(obj) = redisDb.map.get_mut(&key) {
                if let DataType::LIST(list) = &mut obj.data {
                    println!("order given{:?}", values);
                    for value in values {
                        list.push_front(value);
                    }
                    stream
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
            redisDb.map.insert(key, obj);
            stream.write_all(&parse_resp(Resp::Integer(count))).unwrap()
        }
    }

    pub fn r_push(stream: &mut TcpStream, redisDb: &mut RedisDb, key: String, values: Vec<String>) {
        println!("rpush");
        if redisDb.map.contains_key(&key) {
            if let Some(obj) = redisDb.map.get_mut(&key) {
                if let DataType::LIST(list) = &mut obj.data {
                    for value in values {
                        list.push_back(value);
                    }
                    stream
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
            redisDb.map.insert(key, obj);
            stream.write_all(&parse_resp(Resp::Integer(count))).unwrap()
        }
    }
}
