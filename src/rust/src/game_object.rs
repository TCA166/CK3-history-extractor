use std::collections::HashMap;

use crate::{game_state::GameState, structures::{GameObjectDerived}};

#[derive(Debug)]
pub enum SaveFileValue{
    String(String),
    Object(GameObject)
}

impl SaveFileValue {

    /// Create a new Option<&String> from a SaveFileValue
    pub fn as_string(&self) -> Option<&String>{
        match self{
            SaveFileValue::String(s) => Some(s),
            _ => None
        }
    }

    /// Create a new Option<&GameObject> from a SaveFileValue
    pub fn as_object(&self) -> Option<&GameObject>{
        match self{
            SaveFileValue::Object(o) => Some(o),
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

    /// Get the value of an index in the GameObject array
    pub fn get_index(&self, index: usize) -> Option<&SaveFileValue>{
        self.array.get(index)
    }

    /// Get the name of the GameObject
    pub fn get_name(&self) -> &str{ 
        &self.name
    }

    pub fn get_as<T> (&self, key: &str, game_state: &GameState) -> T where T: GameObjectDerived{
        T::from_game_object(self.get(key).unwrap().as_object().unwrap(), game_state)
    }

    pub fn to<T> (&self, game_state: &GameState) -> T where T: GameObjectDerived{
        T::from_game_object(self, game_state)
    }

    /// Push a new value into the GameObject array
    pub fn push(&mut self, value: SaveFileValue){
        self.array.push(value);
    }

    /// Get the length of the GameObject array
    pub fn is_empty(&self) -> bool{
        self.inner.is_empty() && self.array.is_empty()
    }
}
