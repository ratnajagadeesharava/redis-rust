use std::{fmt::format, result};

#[derive(Debug)]
pub enum Resp {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<String>),
    Other(String),
}

pub fn parse_resp(value: Resp) -> Vec<u8> {
    match value {
        Resp::SimpleString(val) => {
            let result = format!("+{val}\r\n");
            result.into_bytes()
        }
        Resp::Error(_) => todo!(),
        Resp::Integer(val) =>{
            let result = format!(":{val}\r\n");
            result.into_bytes()
        },
        Resp::BulkString(val) => {
            let result = format!("${}\r\n{val}\r\n", val.len());
            result.into_bytes()
        }
        Resp::Array(items) => {
            let l = items.len();
            let mut result:String = format!("*{l}\r\n");
            for item in items{
                let bytes = parse_resp(Resp::BulkString(item));
                result += &String::from_utf8(bytes).unwrap();
            }
            result.into_bytes()

        },
        Resp::Other(_) => todo!(),
    }
}

pub fn parse_message(message: &str) -> Resp {
    let n = message.len();
    let chars: Vec<char> = message.chars().collect();
    let mut index = 0;

    match chars[index] {
        '+' => {
            // index += 1;
            let value = &message[1..n - 2];
            Resp::SimpleString(value.to_string())
        }
        '-' => {
            let value = &message[1..n - 2];
            Resp::Error(value.to_string())
        }
        ':' => {
            let value = message[1..n - 2].parse::<i64>().unwrap();
            Resp::Integer(value)
        }
        '$' => {
            let values: Vec<&str> = message[1..n - 2].split("\r\n").collect();
            let value = values[1];
            Resp::BulkString(value.to_string())
        }
        '*' => {
            let values: Vec<&str> = message[1..n - 2].split("\r\n").collect();
            let cmds: Vec<String> = values.into_iter().map(|s| s.to_string()).collect();
            Resp::Array(cmds)
        }
        _ => Resp::Other(message.to_string()),
    }
}
