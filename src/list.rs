use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

pub struct ListNode {
    pub val: String,
    pub next: Option<Rc<RefCell<ListNode>>>,
    pub prev: Option<Rc<RefCell<ListNode>>>,
}

impl ListNode {
    pub fn new(value: String) -> Self {
        Self {
            val: value,
            next: None,
            prev: None,
        }
    }
}

pub struct List {
    pub head: Option<Rc<RefCell<ListNode>>>,
    pub tail: Option<Rc<RefCell<ListNode>>>,
    pub count: i64,
}

impl List {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            count: 0,
        }
    }
    pub fn push_back(&mut self, val: String) {
        self.count += 1;
        let new_node = Rc::new(RefCell::new(ListNode::new(val)));

        match self.tail.take() {
            Some(node) => {
                node.borrow_mut().next = Some(new_node.clone());
                new_node.borrow_mut().prev = Some(node.clone());
                self.tail = Some(new_node.clone());
            }
            None => {
                self.head = Some(new_node.clone());
                self.tail = Some(new_node.clone());
            }
        }
    }

    pub fn push_front(&mut self, val: String) {
        self.count += 1;
        let new_node = Rc::new(RefCell::new(ListNode::new(val)));

        match self.tail.take() {
            Some(node) => {
                node.borrow_mut().prev = Some(new_node.clone());
                new_node.borrow_mut().next = Some(node.clone());
                self.head = Some(new_node.clone());
            }
            None => {
                self.head = Some(new_node.clone());
                self.tail = Some(new_node.clone());
            }
        }
    }

    pub fn pop_back(&mut self) -> Option<Rc<RefCell<ListNode>>> {
        match self.tail.take() {
            Some(node) => {
                match node.borrow_mut().prev.clone() {
                    Some(p) => {
                        p.borrow_mut().next = None;
                        self.tail = Some(p.clone());
                    }
                    None => {
                        self.head = None;
                    }
                }
                node.borrow_mut().prev = None;
                self.count -= 1;
                Some(node)
            }
            None => None,
        }
    }
    pub fn range(&self, start: usize, end: usize) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = self.head.clone();
        let mut index = 0;

        while let Some(node) = current {
            let node_ref = node.borrow();

            if index >= start && index <= end {
                result.push(node_ref.val.clone());
            }

            if index > end {
                break;
            }

            current = node_ref.next.clone();
            index += 1;
        }

        result
    }
}
