use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;

use crate::game_object::{GameObject, SaveFileValue};

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
                    if past_eq {
                        if !val.is_empty() {
                            stack.last_mut().unwrap().insert(key.clone(), SaveFileValue::String(Rc::new(val.clone())));
                            key.clear();
                            val.clear();
                            past_eq = false;
                        }
                    }
                    else {
                        if !key.is_empty() {
                            stack.last_mut().unwrap().push(SaveFileValue::String(Rc::new(key.clone())));
                            key.clear();
                        }
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
                        stack.last_mut().unwrap().insert(inner.get_name().to_string(), SaveFileValue::Object(Rc::new(inner)));
                    }
                }
                '\n' => { // we have reached the end of a line, we check if we have a key value pair
                    if past_eq { // we have a key value pair
                        stack.last_mut().unwrap().insert(key.clone(), SaveFileValue::String(Rc::new(val.clone())));
                        key.clear();
                        val.clear();
                        past_eq = false; // we reset the past_eq flag
                    }
                    else if !key.is_empty(){ // we have just a key { \n key \n }
                        stack.last_mut().unwrap().push(SaveFileValue::String(Rc::new(key.clone())));
                        key.clear();
                    }
                }
                ' ' | '\t' => { //syntax sugar we ignore, most of the time
                    if past_eq && !val.is_empty() { // in case {something=else something=else}
                        stack.last_mut().unwrap().insert(key.clone(), SaveFileValue::String(Rc::new(val.clone())));
                        key.clear();
                        val.clear();
                        past_eq = false;
                    }
                    else if !key.is_empty() && !past_eq{ // in case { something something something } we want to preserve the spaces
                        stack.last_mut().unwrap().push(SaveFileValue::String(Rc::new(key.clone())));
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

mod tests {

    use std::io::Write;

    use tempfile::NamedTempFile;

    use crate::game_object::GameObject;

    #[test]
    fn test_save_file(){
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"
            test={
                test2={
                    test3=1
                }
            }
        ").unwrap();
        let mut save_file = super::SaveFile::new(file.path().to_str().unwrap());
        let object = save_file.next().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.get("test2").unwrap().as_object();
        assert!(test2.is_some());
        let binding = test2.unwrap();
        let test3 = binding.get("test3");
        assert!(test3.is_some());
        assert_eq!(*(test3.unwrap().as_string().unwrap()) , "1".to_string());
    }

    #[test]
    fn test_save_file_array(){
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"
            test={
                test2={
                    1
                    2
                    3
                }
                test3={ 1 2 3}
            }
        ").unwrap();
        let mut save_file = super::SaveFile::new(file.path().to_str().unwrap());
        let object = save_file.next().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.get("test2").unwrap().as_object();
        assert!(test2.is_some());
        let test2_val = test2.unwrap();
        assert_eq!(*(test2_val.get_index(0).unwrap().as_string().unwrap()) , "1".to_string());
        assert_eq!(*(test2_val.get_index(1).unwrap().as_string().unwrap()) , "2".to_string());
        assert_eq!(*(test2_val.get_index(2).unwrap().as_string().unwrap()) , "3".to_string());
        let test3 = object.get("test3").unwrap().as_object();
        assert!(test3.is_some());
        let test3_val = test3.unwrap();
        assert_eq!(*(test3_val.get_index(0).unwrap().as_string().unwrap()) , "1".to_string());
        assert_eq!(*(test3_val.get_index(1).unwrap().as_string().unwrap()) , "2".to_string());
        assert_eq!(*(test3_val.get_index(2).unwrap().as_string().unwrap()) , "3".to_string());
    }

    #[test]
    fn test_weird_syntax(){
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"
            test={
                test2={1=2
                    3=4}
                test3={1 2 
                    3}
                test4={1 2 3}
                test5=42
            }
        ").unwrap();
        let mut save_file = super::SaveFile::new(file.path().to_str().unwrap());
        let object = save_file.next().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.get("test2").unwrap().as_object();
        assert!(test2.is_some());
        let test2_val = test2.unwrap();
        assert_eq!(*(test2_val.get("1").unwrap().as_string().unwrap()) , "2".to_string());
        assert_eq!(*(test2_val.get("3").unwrap().as_string().unwrap()) , "4".to_string());
    }
}
