use std::{cell::RefCell, net::TcpStream, rc::Rc, time::SystemTime};
pub type ClientId = usize;
pub struct Client{
    pub stream:RefCell<TcpStream>,
    pub clientId:ClientId,
    pub blocked:bool,
    pub waiting_key:Option<String>,
    pub waiting_time:Option<SystemTime>
}

