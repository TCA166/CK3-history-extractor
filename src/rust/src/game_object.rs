use std::{cell::Ref, collections::HashMap};

use crate::structures::Shared;

#[derive(Debug)]
pub enum SaveFileValue{
    String(Shared<String>),
    Object(Shared<GameObject>)
}

impl SaveFileValue {

    /// Create a new Option<&String> from a SaveFileValue
    pub fn as_string_ref(&self) -> Option<Ref<'_, String>>{
        match self{
            SaveFileValue::String(s) => Some(s.as_ref().borrow()),
            _ => None
        }
    }

    pub fn as_string(&self) -> Shared<String>{
        match self{
            SaveFileValue::String(s) => s.clone(),
            _ => panic!("Invalid value")
        }
    }

    /// Create a new Option<&GameObject> from a SaveFileValue
    pub fn as_object_ref(&self) -> Option<Ref<'_, GameObject>>{
        match self{
            SaveFileValue::Object(o) => Some(o.as_ref().borrow()),
            _ => None
        }
    }

}

#[derive(Debug)]
pub struct GameObject{
    inner: HashMap<String, SaveFileValue>,
    array: Vec<SaveFileValue>,
    name: String
}

impl GameObject{
    
    /// Create a new GameObject from a name
    pub fn from_name(name: &String) -> GameObject{
        GameObject{
            inner: HashMap::new(),
            name: name.clone(),
            array: Vec::new(),
        }
    }

    /// Create a new GameObject
    pub fn new() -> GameObject{
        GameObject{
            inner: HashMap::new(),
            name: String::new(),
            array: Vec::new(),
        }
    }

    /// Rename the GameObject
    pub fn rename(&mut self, name: String){
        self.name = name;
    }

    /// Insert a new key value pair into the GameObject
    pub fn insert(&mut self, key: String, value: SaveFileValue){
        self.inner.insert(key, value);
    }

    /// Get the value of a key
    pub fn get(&self, key: &str) -> Option<&SaveFileValue>{
        self.inner.get(key)
    }

    pub fn get_string_ref(&self, key: &str) -> Ref<'_, String>{
        self.inner.get(key).unwrap().as_string_ref().unwrap()
    }

    pub fn get_object_ref(&self, key: &str) -> Ref<'_, GameObject>{
        self.inner.get(key).unwrap().as_object_ref().unwrap()
    }

    /// Get the value of an index in the GameObject array
    pub fn get_index(&self, index: usize) -> Option<&SaveFileValue>{
        self.array.get(index)
    }

    /// Get the name of the GameObject
    pub fn get_name(&self) -> &str{ 
        &self.name
    }

    /// Push a new value into the GameObject array
    pub fn push(&mut self, value: SaveFileValue){
        self.array.push(value);
    }

    /// Get the length of the GameObject array
    pub fn is_empty(&self) -> bool{
        self.inner.is_empty() && self.array.is_empty()
    }

    pub fn get_array(&self) -> &Vec<SaveFileValue>{
        &self.array
    }

    pub fn get_keys(&self) -> Vec<String>{
        self.inner.keys().map(|x| x.clone()).collect()
    }
}
