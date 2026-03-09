use std::{cell::RefCell, rc::Rc, string};

use crate::list::List;

pub struct RadixNode {
    pub value: String,
    pub children: Option<Vec<(char, Box<RadixNode>)>>,
    pub data: Option<Vec<String>>,
}

impl RadixNode {
    pub fn new(&mut self) -> Self {
        Self {
            value: "".to_string(),
            children: Some(Vec::new()),
            data: None,
        }
    }
    pub fn add(&mut self, id: String, val: String) {
        let arr: Vec<String> = id.split("-").map(|s| s.to_string()).collect();
        let id = arr[0].clone();
        let seq: usize = arr[1].parse().unwrap();
        match &mut self.children {
            Some(children) => {
                
            }
            None => {
                
            }
        }
    }
}
/*
let mut child = Self {
                    value: id.clone(),
                    children: None,
                    data: Some(Vec::new()),
                };
                if let Some(arr) = &mut child.data {
                    arr.push(val);
                }
                children.push((id.chars().nth(0).unwrap(), Box::new(child)));
 */