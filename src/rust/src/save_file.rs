use std::fs::File;
use std::io::prelude::*;

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

    /// Get the next object in the save file
    pub fn next(&mut self) -> GameObject{
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
                self.file.read(&mut buffer).unwrap();
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
                    if !key.is_empty() {
                        key.split(" ").for_each(|x| {
                            stack.last_mut().unwrap().push(game_objects::SaveFileValue::String(x.to_string()));
                        });
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
                    if past_eq { //we have a key value pair
                        stack.last_mut().unwrap().insert(key.clone(), SaveFileValue::String(val.clone()));
                        key.clear();
                        val.clear();
                        past_eq = false; // we reset the past_eq flag
                    }
                    else{ // we have just a key
                        stack.last_mut().unwrap().push(SaveFileValue::String(key.clone()));
                        key.clear();
                    }
                }
                ' ' | '\t' => {} //syntax sugar we ignore
                '=' => { // we have an assignment
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
        return object;
    }
}
