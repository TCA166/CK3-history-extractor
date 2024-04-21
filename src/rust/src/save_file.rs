use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

mod game_objects;
use game_objects::{GameObject, SaveFileValue};

/// A struct that represents a ck3 save file
pub struct SaveFile{
    file: File
}

impl SaveFile{

    /// Create a new SaveFile instance
    pub fn new(filename: &str) -> SaveFile{
        SaveFile{
            file: File::open(filename).unwrap(),
        }
    }

    pub fn reset(&mut self){
        self.file.seek(SeekFrom::Start(0)).unwrap();
    }

}

impl Iterator for SaveFile{

    type Item = GameObject;

    /// Get the next object in the save file
    fn next(&mut self) -> Option<GameObject>{
        // storage
        let mut key = String::new();
        let mut val = String::new();
        let mut past_eq = false; // we use this flag to determine if we are parsing a key or a value
        let mut depth = 0; // how deep we are in the object tree
        let mut root_name:String = String::new();
        let mut object:GameObject = GameObject::new();
        //initialize the object stack
        let mut stack: Vec<GameObject> = Vec::new();
        stack.push(object);
        //initialize the key stack
        loop{
            let c: char;
            { // we read a single byte from the file
                let mut buffer = [0; 1];
                let ret = self.file.read(&mut buffer);
                if ret.is_err(){
                    break;
                }
                c = buffer[0] as char;
            }
            match c{ // we parse the byte
                '{' => { // we have a new object, we push a new hashmap to the stack
                    if depth == 0 {
                        root_name.clone_from(&key.clone());
                    }
                    depth += 1;
                    stack.push(GameObject::from_name(&key));
                    key.clear();
                    past_eq = false;
                }
                '}' => { // we have reached the end of an object
                    if !key.is_empty() && !past_eq { //the only case where this is possible is if we have an array
                        stack.last_mut().unwrap().push(game_objects::SaveFileValue::String(key.clone()));
                        key.clear();
                    }
                    depth -= 1;
                    if depth == 0{ // we have reached the end of the object we are parsing, we return the object
                        break;
                    }
                    else if depth < 0 { // we have reached the end of the file
                        panic!("Depth is negative");
                    }
                    else{ // we pop the inner object and insert it into the outer object
                        let inner = stack.pop().unwrap();
                        stack.last_mut().unwrap().insert(inner.get_name().to_string(), SaveFileValue::Object(inner));
                    }
                }
                '\n' => { // we have reached the end of a line, we check if we have a key value pair
                    if past_eq { // we have a key value pair
                        stack.last_mut().unwrap().insert(key.clone(), SaveFileValue::String(val.clone()));
                        key.clear();
                        val.clear();
                        past_eq = false; // we reset the past_eq flag
                    }
                    else if !key.is_empty(){ // we have just a key { \n key \n }
                        stack.last_mut().unwrap().push(SaveFileValue::String(key.clone()));
                        key.clear();
                    }
                }
                ' ' | '\t' => { //syntax sugar we ignore, most of the time
                    if past_eq && !val.is_empty() { // in case {something=else something=else}
                        stack.last_mut().unwrap().insert(key.clone(), SaveFileValue::String(val.clone()));
                        key.clear();
                        val.clear();
                        past_eq = false;
                    }
                    else if !key.is_empty() && !past_eq{ // in case { something something something } we want to preserve the spaces
                        stack.last_mut().unwrap().push(SaveFileValue::String(key.clone()));
                        key.clear();
                    }
                } 
                '=' => {
                    // if we have an assignment, we toggle adding from key to value
                    past_eq = true;
                }
                _ => { //the main content we append to the key or value
                    if past_eq{
                        val.push(c);
                    }else{
                        key.push(c);
                    }
                }
            }
        }
        object = stack.pop().unwrap();
        object.rename(root_name.clone());
        if object.is_empty(){
            return None;
        }
        return Some(object);
    }
}
