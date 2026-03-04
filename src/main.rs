#![allow(unused_imports)]
use std::{
    collections::VecDeque,
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::net::SocketAddr,
    str::from_utf8,
};

use bytes::buf;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment the code below to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let mut queue = VecDeque::<TcpStream>::new();
    let mut clients = Vec::<TcpStream>::new();
    listener
        .set_nonblocking(true)
        .expect("non blocking is not possible");
    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                clients.push(stream);
            }
            Err(error) => {
                // println!("{:?}",error);
                // break;
            }
        }
        for mut _stream in &clients {
            let mut buffer = [0; 1024];

            let bytes_read = _stream.read(&mut buffer).expect("stream is not read");
            // println!("sadasd");
            let message = from_utf8(&buffer[..bytes_read]).unwrap();
            let count = message.matches("PING").count();
            if bytes_read != 0 {
                for _ in 0..count {
                    _stream.write_all(b"+PONG\r\n").unwrap();
                }
                // println!("PONG");
            }
        }
    }
}

