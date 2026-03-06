#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
mod list;
mod redisCommand;
mod redisDb;
mod redisObject;
mod redisServer;
mod resp;
use bytes::buf;
use regex::Regex;
use regex_split::RegexSplit;
use resp::Resp;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    f32::consts::E,
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::net::SocketAddr,
    str::from_utf8,
    time::{Duration, SystemTime, SystemTimeError},
};

use crate::{redisDb::RedisDb, redisServer::RedisServer, resp::parse_message};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let mut redisDb = RedisDb::new(); //lets think there is no user specific data
    let mut clients = Vec::<(TcpStream)>::new();

    listener
        .set_nonblocking(true)
        .expect("non blocking is not possible");
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(true).unwrap();
                clients.push(stream);
                // println!("{:?}",clients);
            }
            Err(_) => {}
        }

        for mut _stream in &mut clients {
            
            RedisServer::execute_stream(&mut redisDb, &mut _stream);
        }
    }
}


