use std::{
    io::{Read, Write},
    net::TcpStream,
    os::unix::raw::pid_t,
    str::from_utf8,
    time::{Duration, SystemTime},
};

use crate::{
    list::List, redisCommand::{RedisCommand, array_to_command}, redisDb::RedisDb, redisObject::{DataType, RedisObject}, resp::{Resp, parse_message, parse_resp}
};

pub struct RedisServer;
impl RedisServer {
    pub fn execute(cmd: RedisCommand, redisDb: &mut RedisDb, stream: &mut TcpStream) {
        match cmd {
            RedisCommand::Set(key, value, ttl) => {
                Self::set_command(stream, redisDb, key, value, ttl)
            }
            RedisCommand::Get(key) => Self::get_command(stream, redisDb, key),
            RedisCommand::RPush(key, value) => Self::r_push(stream,redisDb,key,value),
            RedisCommand::Echo(value) => Self::echo(stream, value),
            RedisCommand::Unkown => todo!(),
            RedisCommand::Ping => Self::ping(stream),
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
                        println!("{:?}",message);
                        let command_array: Vec<String> =
                            array_iter.filter(|val: &String| val.len() != 0).collect();
                            println!("{:?}",command_array);
                        let redisCommnad = array_to_command(&command_array);
                        
                        Self::execute(redisCommnad, redisDb, stream);
                    }
                }
            }
            Err(_) => {}
        }
    }

    pub fn r_push( stream: &mut TcpStream,
        redisDb: &mut RedisDb,
        key: String,
        values: Vec<String>){
            if redisDb.map.contains_key(&key){

                if let Some(obj) = redisDb.map.get_mut(&key) {
                    if let DataType::LIST(list) = &mut obj.data{
                        for value in values{
                            list.push_back(value);
                        }
                        stream.write_all(&parse_resp(Resp::Integer(list.count))).unwrap()
                    }
                }
            }
            else{
                let mut  list = List::new();
                for value in values{
                list.push_back(value);
                }
                let obj = RedisObject{
                    data:DataType::LIST(list)
                };
                redisDb.map.insert(key, obj);
                stream.write_all(&parse_resp(Resp::Integer(1))).unwrap()
            }
            
        }
}
