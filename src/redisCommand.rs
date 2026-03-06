use std::{
    collections::{HashMap, btree_map::Values},
    io::Write,
    net::TcpStream,
    time::{Duration, SystemTime},
};

use crate::{
    redisDb::RedisDb,
    redisObject::{DataType, RedisObject},
    resp::{Resp, parse_resp},
};

#[derive(Debug, Clone)]
pub enum RedisCommand {
    Set(String, String, Option<u64>),
    Get(String),
    Ping,
    RPush(String, Vec<String>),
    Echo(String),
    Unkown,
    LRANGE(String,i32,i32),
     LPush(String, Vec<String>)
}

type CommandFn = fn();
//["5", "SET", "mykey", "hello", "PX", "5000"]
pub fn array_to_command(command_array: &Vec<String>) -> RedisCommand {
    let mut index = 0;
    let n = command_array.len();
    let mut redisCommand = RedisCommand::Unkown;
    loop {
        let cmd = &command_array[index];
        match cmd.as_str() {
            "SET" => {
                index += 2;
                let key = command_array[index].clone();
                index += 2;
                let value = command_array[index].clone();
                index += 4;
                if index <= n - 1 {
                    let ttl: u64 = command_array[index].as_str().parse().unwrap();
                    redisCommand = RedisCommand::Set(key, value, Some(ttl));
                } else {
                    redisCommand = RedisCommand::Set(key, value, None);
                }
            }
            "GET" => redisCommand = RedisCommand::Get(command_array[index + 2].clone()),
            "PING" => redisCommand = RedisCommand::Ping,
            "ECHO" => {
                index+=2;
                println!("{:?}",command_array[index]);
                redisCommand = RedisCommand::Echo(command_array[index ].clone())},
            "RPUSH" => {
                index += 2;
                // println!("{}",command_array[index]);
                let key = command_array[index].clone();
                let mut values = Vec::<String>::new();
                index += 2;
                for i in (index..n).step_by(2){
                    values.push(command_array[i].clone());
                }

                redisCommand = RedisCommand::RPush(key, values);
            }
            "LPUSH" => {
                index += 2;
                // println!("{}",command_array[index]);
                let key = command_array[index].clone();
                let mut values = Vec::<String>::new();
                index += 2;
                for i in (index..n).step_by(2){
                    values.push(command_array[i].clone());
                }

                redisCommand = RedisCommand::LPush(key, values);
            }
            "LRANGE"=>{
                index+=2;
                let key = command_array[index].clone();
                index+=2;
                let start:i32 = command_array[index].clone().parse().unwrap();
                index+=2;
                let end:i32 = command_array[index].clone().parse().unwrap();
                redisCommand = RedisCommand::LRANGE(key, start, end)
            }
            _ => {
                index += 1;
                continue;
            }
        }
        break;
    }

    redisCommand
}
