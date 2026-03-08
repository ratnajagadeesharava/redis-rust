#![allow(unused_imports)]
#![allow(warnings)]
#![allow(unused_variables)]
#![allow(unused_mut)]
mod client;
mod list;
mod redisCommand;
mod redisDb;
mod redisObject;
mod redisServer;
mod resp;
use resp::Resp;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    f32::consts::E,
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::net::SocketAddr,
    str::from_utf8,
    time::{Duration, SystemTime, SystemTimeError},
};

use crate::{
    client::{Client, ClientId},
    redisDb::RedisDb,
    redisServer::RedisServer,
    resp::parse_message,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    let mut server: RedisServer = RedisServer {
        client_map: HashMap::new(),
        redis_db: RedisDb::new(), //lets think there is no user specific data
    };
    let mut clients = Vec::<ClientId>::new();

    listener
        .set_nonblocking(true)
        .expect("non blocking is not possible");

    let mut client_id: ClientId = 0;

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(true).unwrap();
                let mut client = Client {
                    stream: RefCell::new(stream),
                    clientId: client_id,
                    blocked: false,
                    waiting_key: None,
                    waiting_time: None,
                };
                server.client_map.insert(client_id, client);
                clients.push(client_id);

                client_id += 1;
            }
            Err(_) => {}
        }

        for clientId in &mut clients {
            server.execute_stream(*clientId);
            if let Some(client) = server.client_map.get_mut(clientId) {
                if client.blocked {
                    if let Some(waiting_time) = client.waiting_time {
                        if waiting_time < SystemTime::now() {
                            client.waiting_key = None;
                            client.waiting_time = None;
                            client.blocked = false;
                            client.stream.borrow_mut().write_all(b"*-1\r\n").unwrap();
                        }
                    }
                }
            }
        }
    }
}
