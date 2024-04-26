use std::{cell::RefCell, fs::File};
use std::io::prelude::*;
use std::rc::Rc;

use crate::game_object::{GameObject, SaveFileValue};

/// A function that reads a single character from a file
/// 
/// ## Returns
/// 
/// The character read or None if the end of the file is reached
fn fgetc(file: &mut File) -> Option<char>{ // Shoutout to my C literate homies out there
    let mut buffer = [0; 1];
    let ret = file.read(&mut buffer);
    if ret.is_err(){
        return None;
    }
    return Some(buffer[0] as char);
}

pub struct Section{
    name: String,
    file:Option<File>
}

impl Section{
    fn new(name: &str, file: File) -> Section{
        Section{
            name: name.to_string(),
            file: Some(file)
        }
    }

    /// Get the name of the section
    pub fn get_name(&self) -> &str{
        &self.name
    }

    fn invalidate(&mut self){
        self.file = None;
    }

    /// Convert the section to a GameObject and invalidate it
    pub fn to_object(&mut self) -> Option<GameObject>{
        if self.file.is_none(){
            panic!("Invalid section");
        }
        let file = &mut self.file.as_mut().unwrap();
        // storage
        let mut key = String::new();
        let mut val = String::new();
        let mut past_eq = false; // we use this flag to determine if we are parsing a key or a value
        let mut depth = 0; // how deep we are in the object tree
        let mut object:GameObject = GameObject::new();
        //initialize the object stack
        let mut stack: Vec<GameObject> = Vec::new();
        stack.push(object);
        //initialize the key stack
        loop{
            let ret = fgetc(file);
            if ret.is_none(){
                break;
            }
            let c = ret.unwrap();
            match c{ // we parse the byte
                '{' => { // we have a new object, we push a new hashmap to the stack
                    depth += 1;
                    stack.push(GameObject::from_name(&key));
                    key.clear();
                    past_eq = false;
                }
                '}' => { // we have reached the end of an object
                    if past_eq {
                        if !val.is_empty() {
                            stack.last_mut().unwrap().insert(key.clone(), SaveFileValue::String(Rc::new(RefCell::new(val.clone()))));
                            key.clear();
                            val.clear();
                            past_eq = false;
                        }
                    }
                    else {
                        if !key.is_empty() {
                            stack.last_mut().unwrap().push(SaveFileValue::String(Rc::new(RefCell::new(key.clone()))));
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
                        stack.last_mut().unwrap().insert(inner.get_name().to_string(), SaveFileValue::Object(Rc::new(RefCell::new(inner))));
                    }
                }
                '\n' => { // we have reached the end of a line, we check if we have a key value pair
                    if past_eq { // we have a key value pair
                        stack.last_mut().unwrap().insert(key.clone(), SaveFileValue::String(Rc::new(RefCell::new(val.clone()))));
                        key.clear();
                        val.clear();
                        past_eq = false; // we reset the past_eq flag
                    }
                    else if !key.is_empty(){ // we have just a key { \n key \n }
                        stack.last_mut().unwrap().push(SaveFileValue::String(Rc::new(RefCell::new(key.clone()))));
                        key.clear();
                    }
                }
                ' ' | '\t' => { //syntax sugar we ignore, most of the time
                    if past_eq && !val.is_empty() { // in case {something=else something=else}
                        stack.last_mut().unwrap().insert(key.clone(), SaveFileValue::String(Rc::new(RefCell::new(val.clone()))));
                        key.clear();
                        val.clear();
                        past_eq = false;
                    }
                    else if !key.is_empty() && !past_eq{ // in case { something something something } we want to preserve the spaces
                        stack.last_mut().unwrap().push(SaveFileValue::String(Rc::new(RefCell::new(key.clone()))));
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
        object.rename(self.name.clone());
        self.invalidate();
        if object.is_empty(){
            return None;
        }
        return Some(object);
    }

    /// Skip the current section and invalidate it
    pub fn skip(&mut self){
        if self.file.is_none(){
            panic!("Invalid section");
        }
        let mut depth = 0;
        let file = &mut self.file.as_mut().unwrap();
        loop{
            let ret = fgetc(file);
            if ret.is_none(){
                break;
            }
            let c = ret.unwrap();
            match c{
                '{' => {
                    depth += 1;
                }
                '}' => {
                    depth -= 1;
                    if depth == 0{
                        break;
                    }
                }
                _ => {}
            }
        }
        self.invalidate();
    }
}

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

    type Item = Section;

    /// Get the next object in the save file
    fn next(&mut self) -> Option<Section>{
        let mut key = String::new();
        let file = &mut self.file;
        loop{
            let ret = fgetc(file);
            if ret.is_none(){
                return None;
            }
            let c = ret.unwrap();
            match c{
                '{' | '}' | '"' => {
                    panic!("Unexpected character");
                }
                '=' => {
                    break;
                }
                '\n' => {
                    key.clear();
                }
                ' ' | '\t' => {
                    continue;
                }
                _ => {
                    key.push(c);
                }
            }
            
        }
        return Some(Section::new(&key, self.file.try_clone().unwrap()));
    }
}

mod tests {

    #[allow(unused_imports)]
    use std::io::Write;

    #[allow(unused_imports)]
    use tempfile::NamedTempFile;

    #[allow(unused_imports)]
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
        let object = save_file.next().unwrap().to_object().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.get_object_ref("test2");
        let test3 = test2.get_string_ref("test3");
        assert_eq!(*(test3) , "1".to_string());
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
        let object = save_file.next().unwrap().to_object().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.get_object_ref("test2");
        let test2_val = test2;
        assert_eq!(*(test2_val.get_index(0).unwrap().as_string_ref().unwrap()) , "1".to_string());
        assert_eq!(*(test2_val.get_index(1).unwrap().as_string_ref().unwrap()) , "2".to_string());
        assert_eq!(*(test2_val.get_index(2).unwrap().as_string_ref().unwrap()) , "3".to_string());
        let test3 = object.get_object_ref("test3");
        let test3_val = test3;
        assert_eq!(*(test3_val.get_index(0).unwrap().as_string_ref().unwrap()) , "1".to_string());
        assert_eq!(*(test3_val.get_index(1).unwrap().as_string_ref().unwrap()) , "2".to_string());
        assert_eq!(*(test3_val.get_index(2).unwrap().as_string_ref().unwrap()) , "3".to_string());
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
        let object = save_file.next().unwrap().to_object().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.get_object_ref("test2");
        assert_eq!(*(test2.get_string_ref("1")) , "2".to_string());
        assert_eq!(*(test2.get_string_ref("3")) , "4".to_string());
    }

    #[test]
    fn test_array_syntax(){
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"
            test={
                test2={ 1 2 3 }
            }
        ").unwrap();
        let mut save_file = super::SaveFile::new(file.path().to_str().unwrap());
        let object = save_file.next().unwrap().to_object().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.get_object_ref("test2");
        assert_eq!(*(test2.get_index(0).unwrap().as_string_ref().unwrap()) , "1".to_string());
        assert_eq!(*(test2.get_index(1).unwrap().as_string_ref().unwrap()) , "2".to_string());
        assert_eq!(*(test2.get_index(2).unwrap().as_string_ref().unwrap()) , "3".to_string());
    }
}
