use core::time;
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
    LRANGE(String, i32, i32),
    LPush(String, Vec<String>),
    LLEN(String),
    LPOP(String, i32),
    BLPOP(String, i32),
    TYPE(String),
    XADD(String, String, Vec<(String, String)>),
}

type CommandFn = fn();
pub fn array_to_command(command_array: &Vec<String>) -> RedisCommand {
    let args: Vec<&str> = command_array
        .iter()
        .skip(1)
        .step_by(2)
        .map(String::as_str)
        .collect();
    match args.as_slice() {
        ["PING"] => RedisCommand::Ping,
        ["GET", key] => RedisCommand::Get(key.to_string()),
        ["SET", key, value] => RedisCommand::Set(key.to_string(), value.to_string(), None),
        ["SET", key, value, "PX", ttl] => RedisCommand::Set(
            key.to_string(),
            value.to_string(),
            Some(ttl.parse().unwrap()),
        ),

        ["LLEN", key] => RedisCommand::LLEN(key.to_string()),

        ["ECHO", msg] => RedisCommand::Echo(msg.to_string()),

        ["LPOP", key] => RedisCommand::LPOP(key.to_string(), 1),

        ["LPOP", key, count] => RedisCommand::LPOP(key.to_string(), count.parse().unwrap()),

        ["BLPOP", key, timeout] => {
            let t = timeout.parse::<f64>().unwrap() * 1000.0;
            RedisCommand::BLPOP(key.to_string(), t as i32)
        }

        ["RPUSH", key, rest @ ..] => RedisCommand::RPush(
            key.to_string(),
            rest.iter().map(|v| v.to_string()).collect(),
        ),

        ["LPUSH", key, rest @ ..] => RedisCommand::LPush(
            key.to_string(),
            rest.iter().map(|element| element.to_string()).collect(),
        ),
        ["XADD", key, id, rest @ ..] => {
            let mut key_value = Vec::<(String, String)>::new();

            for pair in rest.iter().skip(1).step_by(2).collect::<Vec<_>>().chunks(2) {
                key_value.push((pair[0].to_string(), pair[1].to_string()));
            }

            RedisCommand::XADD(key.to_string(), id.to_string(), key_value)
        }
        ["TYPE", key] => RedisCommand::TYPE(key.to_string()),
        _ => RedisCommand::Unkown,
    }
}
