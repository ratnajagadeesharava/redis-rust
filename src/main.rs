#![allow(unused_imports)]
use std::{
    collections::VecDeque,
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::net::SocketAddr,
    str::from_utf8,
};

use bytes::buf;
use regex::Regex;
#[derive(Debug)]
enum Resp{
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<Resp>),
    Other(String)
}
fn extract_RESP_Comands(message:&str)->Resp{
    let chars:Vec<char> = message.chars().collect();
    let n = message.len();
    match chars[0]{
        '+'=>{
            Resp::SimpleString(String::from(&message[1..n-2]))
        }
        '-'=>{
           Resp::Error(String::from(&message[1..n-2]))
        }
        ':'=>{
            Resp::Integer((&message[1..n]).parse::<i64>().unwrap())
        }
        '$'=>{
           Resp::BulkString(String::from(&message[1..n-2]))
        }
        '*'=>{
            let mut respArray =Vec::<Resp>::new(); 
            let re = Regex::new(r"(\+.*\r\n)|(\-.*\r\n)|(\:[0-9]+\r\n)|(\$[0-9]+\r\n.*\r\n)|(\*[0-9]+\r\n)");
            let messages:Vec<&str> = message.split("\r\n").collect();
            let arrLen = messages[0].parse::<usize>();
            for i in 1..messages.len(){
                respArray.push(extract_RESP_Comands(messages[i]));
            }
            Resp::Array(respArray)
        }
        _=>{
            Resp::Other(String::from(&message[1..n]))
            
        }
    }
}

fn handle_stream(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            let message = from_utf8(&buffer[..bytes_read]).unwrap();
            // let messages: Vec<String> = Vec::<String>::new();
            // println!("{message}");
            let messages = extract_RESP_Comands(message);
            println!("{:?}",messages);
            let count: usize = message.matches("PING\r\n").count();
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
