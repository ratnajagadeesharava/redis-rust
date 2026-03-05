#![allow(unused_imports)]
use std::{
    collections::VecDeque,
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::net::SocketAddr,
    str::from_utf8,
};

use bytes::buf;

fn handle_stream(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            let message = from_utf8(&buffer[..bytes_read]).unwrap();
            println!("{message}");
            let count = message.matches("PING\r\n").count();
            if bytes_read != 0 {
                for _ in 0..count {
                    stream.write_all(b"+PONG\r\n").unwrap();
                }
            }
        }
        Err(_) => {}
    }
}

fn main() {

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let mut clients = Vec::<TcpStream>::new();
    listener
        .set_nonblocking(true)
        .expect("non blocking is not possible");
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(true).unwrap();
                clients.push(stream);
            }
            Err(_) => {
                
            }
        }
        for _stream in &mut clients {
            handle_stream(_stream);
        }
    }
}
