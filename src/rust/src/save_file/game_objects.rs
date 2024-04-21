use std::collections::HashMap;


#[derive(Debug)]
pub enum SaveFileValue{
    String(String),
    Object(GameObject)
}

#[derive(Debug)]
pub struct GameObject{
    inner: HashMap<String, SaveFileValue>,
    array: Vec<SaveFileValue>,
    name: String
}

impl GameObject{
    
    pub fn from_name(name: &String) -> GameObject{
        GameObject{
            inner: HashMap::new(),
            name: name.clone(),
            array: Vec::new(),
        }
    }

    pub fn new() -> GameObject{
        GameObject{
            inner: HashMap::new(),
            name: String::new(),
            array: Vec::new(),
        }
    }

    pub fn rename(&mut self, name: String){
        self.name = name;
    }

    pub fn insert(&mut self, key: String, value: SaveFileValue){
        self.inner.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&SaveFileValue>{
        self.inner.get(key)
    }

    pub fn get_index(&self, index: usize) -> Option<&SaveFileValue>{
        self.array.get(index)
    }

    pub fn get_name(&self) -> &str{ 
        &self.name
    }

    pub fn push(&mut self, value: SaveFileValue){
        self.array.push(value);
    }

    pub fn is_empty(&self) -> bool{
        self.inner.is_empty() && self.array.is_empty()
    }
}