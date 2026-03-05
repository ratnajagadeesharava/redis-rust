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
use regex_split::RegexSplit;
#[derive(Debug)]
enum Resp {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<Resp>),
    Other(String),
}
fn extract_RESP_Comands(message: &str) -> Resp {
    let chars: Vec<char> = message.chars().collect();
    let n = message.len();
    match chars[0] {
        '+' => Resp::SimpleString(String::from(&message[1..n - 2])),
        '-' => Resp::Error(String::from(&message[1..n - 2])),
        ':' => Resp::Integer((&message[1..n - 2]).parse::<i64>().unwrap()),
        '$' => {
            let messages: Vec<&str> = message.split("\r\n").collect();

            Resp::BulkString(String::from(messages[1]))
        }
        '*' => {
            let mut respArray = Vec::<Resp>::new();
            let re = Regex::new(
                r"(\+.*\r\n)|(\-.*\r\n)|(\:[0-9]+\r\n)|(\$[0-9]+\r\n.*\r\n)|(\*[0-9]+\r\n)",
            )
            .unwrap();

            let messages: Vec<&str> = re.split_inclusive(&message[0..n]).collect();
            println!("{:?}", messages);
            let l = messages.len();
            for i in 1..l {
                if messages[i].len() > 0 {
                    respArray.push(extract_RESP_Comands(messages[i]));
                }
            }
            Resp::Array(respArray)
        }
        _ => Resp::Other(String::from(&message[1..n])),
    }
}
fn execute_redis_command(command:String,arguments:Vec<String>){

}
fn execute_commands(cmd: Resp){
    match cmd{
        Resp::SimpleString(command) => todo!(),
        Resp::Error(_) => todo!(),
        Resp::Integer(_) => todo!(),
        Resp::BulkString(_) => todo!(),
        Resp::Array(resps) => todo!(),
        Resp::Other(_) => todo!(),
    }

}
fn handle_stream(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            if bytes_read != 0 {
                let message = from_utf8(&buffer[..bytes_read]).unwrap();

                let cmd: Resp = extract_RESP_Comands(message);
                execute_commands(cmd);
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
            Err(_) => {}
        }
        for _stream in &mut clients {
            handle_stream(_stream);
        }
    }
}
