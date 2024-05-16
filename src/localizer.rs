use std::collections::HashMap;
use std::{fs, mem};
use std::path::{Path, PathBuf};

use crate::game_object::GameString;
use crate::types::Wrapper;

/// A function that demangles a generic name.
/// It will replace underscores with spaces and capitalize the first letter.
fn demangle_generic(input:&str) -> String{
    let mut s = input.replace("_", " ");
    let bytes = unsafe { s.as_bytes_mut() };
    bytes[0] = bytes[0].to_ascii_uppercase();
    s
}

/// An object that localizes strings.
/// It reads localization data from a directory and provides localized strings.
/// If the localization data is not found, it will demangle the key using an algorithm that tries to approximate the intended text
pub struct Localizer{
    data: Option<HashMap<String, GameString>>
}

impl Localizer{
    pub fn new(localization_src_path:Option<String>) -> Self{
        let mut hmap:Option<HashMap<String, GameString>> = None;
        if localization_src_path.is_some() {
            let path = localization_src_path.unwrap();
            // get every file in the directory and subdirectories
            let mut data: HashMap<String, GameString> = HashMap::new();
            let path = Path::new(&path);
            if path.is_dir() {
                // a stack to keep track of the directories
                let mut stack: Vec<PathBuf> = Vec::new();
                stack.push(PathBuf::from(path));
                // a vector to keep track of all the files
                let mut all_files: Vec<PathBuf> = Vec::new();
                while !stack.is_empty() {
                    let entry = stack.pop().unwrap();
                    if let Ok(entries) = fs::read_dir(entry) {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                if let Ok(file_type) = entry.file_type() {
                                    if file_type.is_dir() {
                                        stack.push(entry.path());
                                    } else if entry.file_name().to_str().unwrap().ends_with(".yml"){
                                        all_files.push(entry.path());
                                    }
                                }
                            }
                        }
                    }
                }
                // having gone through all the directories, we can now read the files
                for entry in all_files {
                    // read the file to string
                    let contents = fs::read_to_string(entry).unwrap();
                    //The thing here is that these 'yaml' files are... peculiar. rust_yaml doesn't seem to be able to parse them correctly
                    //so we doing the thing ourselves :)

                    //parse the 'yaml' file
                    let mut key = String::new();
                    let mut value = String::new();
                    let mut past = false;
                    let mut quotes = false;
                    for char in contents.chars(){
                        match char{
                            ' ' | '\t' => {
                                if quotes {
                                    value.push(char);
                                }
                            },
                            '\n' => {
                                if past && !quotes && !value.is_empty(){
                                    data.insert(mem::take(&mut key), GameString::wrap(mem::take(&mut value)));
                                }
                                past = false;
                                quotes = false;
                            }
                            ':' => {
                                past = true;
                            }
                            '"' => {
                                quotes = !quotes;
                            }
                            _ => {
                                if past {
                                    if quotes {
                                        value.push(char);
                                    }
                                } else {
                                    key.push(char);
                                }
                            }
                        }
                    }
                }
                //TODO resolve the localization functions
                hmap = Some(data);
            }
        }
        Localizer{
            data: hmap
        }
    }

    /// Localizes a string.
    pub fn localize(&self, key: &str) -> GameString{
        if self.data.is_none(){
            return GameString::wrap(demangle_generic(key))
        }
        let data = self.data.as_ref().unwrap();
        if data.contains_key(key){
            return data.get(key).unwrap().clone();
        }
        //println!("Key not found: {}", key);
        GameString::wrap(demangle_generic(key))
    }
}
