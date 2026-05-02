use crate::protocol::Data;
use std::collections::HashMap;

pub type DefaultStorage = HashMap<String, Box<Data>>;

pub trait Storage {
    fn get_data(&self, key: &str) -> Option<Box<Data>>;
    fn set_data(&mut self, key: impl Into<String>, value: Box<Data>);
}

impl Storage for HashMap<String, Box<Data>> {
    fn get_data(&self, key: &str) -> Option<Box<Data>> {
        self.get(key).map(|d| d.clone())
    }
    fn set_data(&mut self, key: impl Into<String>, value: Box<Data>) {
        self.insert(key.into(), value);
    }
}