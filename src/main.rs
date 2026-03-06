#![allow(unused_imports)]
use std::{
    collections::VecDeque, f32::consts::E, io::{ErrorKind, Read, Write}, net::{TcpListener, TcpStream}, os::unix::net::SocketAddr, str::from_utf8
};

use bytes::buf;
use regex::Regex;
use regex_split::RegexSplit;
#[derive(Debug)]
enum Resp<'a> {
    SimpleString(&'a str),
    Error(&'a str),
    Integer(i64),
    BulkString(&'a str),
    Array(Vec<&'a str>),
    Other(&'a str),
}


fn parse_message(message: &str) -> Resp {
    let n = message.len();
    let chars: Vec<char> = message.chars().collect();
    let mut index = 0;

    match chars[index] {
        '+' => {
            // index += 1;
            let value = &message[1..n - 2];
            Resp::SimpleString(value)
        }
        '-' => {
            let value = &message[1..n - 2];
            Resp::Error(value)
        }
        ':' => {
            let value = message[1..n - 2].parse::<i64>().unwrap();
            Resp::Integer(value)
        }
        '$' => {
            let values: Vec<&str> = message[1..n - 2].split("\r\n").collect();
            let value = values[1];
            Resp::BulkString(value)
        }
        '*' => {
            let values: Vec<&str> = message[1..n - 2].split("\r\n").collect();
            Resp::Array(values)
        }
        _ => Resp::Other(message),
    }
}

fn handle_stream(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            if bytes_read != 0 {
                let message = from_utf8(&buffer[..bytes_read]).unwrap();
                let resp = parse_message(message);
                // if let Resp::Array(value) = resp{
                    
                // }
                match resp{
                    Resp::SimpleString(_) => todo!(),
                    Resp::Error(_) => todo!(),
                    Resp::Integer(_) => todo!(),
                    Resp::BulkString(value) => {
                        let count = message.matches("PING").count();
                        for _ in 0..count {
                            stream.write_all(b"+PONG\r\n").unwrap();
                        }
                    },
                    Resp::Array(value) =>{
                        let l = value.len();
                    let command = value[2];
                    if command == "ECHO"{
                        let s = format!("{}\r\n{}\r\n",value[3],value[4]);
                        stream.write_all(s.as_bytes());
                    }
                    },
                    Resp::Other(_) => todo!(),
                }
              
            }
        }
        Err(_) => {}
    }
}
enum Command{
    Echo(String),
    Ping,
    SET(String,String),
    GET(String)
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
            Err(_) => {}
        }
        for _stream in &mut clients {
            handle_stream(_stream);
        }
    }
}
