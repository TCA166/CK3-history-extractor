use std::collections::HashMap;
use std::{fs, mem};
use std::path::{Path, PathBuf};

use crate::game_object::GameString;
use crate::types::Wrapper;

/// A function that demangles a generic name.
/// It will replace underscores with spaces and capitalize the first letter.
fn demangle_generic(input:&str) -> String{
    let mut s = input.trim_start_matches("dynn_").trim_start_matches("nick_").trim_end_matches("_perk").trim_start_matches("death_")
    .trim_start_matches("tenet_").trim_start_matches("doctrine_")
    .trim_start_matches("ethos_").trim_start_matches("heritage_").trim_start_matches("language_").trim_start_matches("martial_custom_").trim_start_matches("tradition_")
    .trim_start_matches("e_").trim_start_matches("k_").trim_start_matches("d_").trim_start_matches("c_").trim_start_matches("b_")
    .trim_end_matches("_name").replace("_", " ");
    let bytes = unsafe { s.as_bytes_mut() };
    bytes[0] = bytes[0].to_ascii_uppercase();
    s
}

/// A function that handles the stack of function calls.
/// It will replace characters from start to end in result according to the functions and arguments in the stack.
fn handle_stack(stack:Vec<(String, Vec<String>)>, start:usize, end:&mut usize, result:&mut String){
    //TODO add more handling
    match stack.len() {
        2 => {
            if stack[0].0 == "GetTrait" && stack[1].0 == "GetName"{
                let l = stack[0].1[0].len();
                result.replace_range(start..*end, stack[0].1[0].as_str().trim_matches('\''));
                // move end to the end of the string
                *end = start + l;
            }
        }
        _ => {
            let replace:String;
            if stack.len() > 0 && stack[0].1.len() > 0{
                replace = stack[0].1[0].clone();
            }
            else{
                replace = "".to_owned();
            }
            result.replace_range(start..*end, &replace);
            *end = start;
        }
    }
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
                                    //Removing trait_? good idea because the localisation isnt consistent enough with trait names
                                    //Removing _name though... controversial. Possibly a bad idea
                                    //MAYBE only do this in certain files
                                    key = key.trim_start_matches("trait_").trim_end_matches("_name").to_string();
                                    data.insert(mem::take(&mut key), GameString::wrap(mem::take(&mut value)));
                                }
                                else{
                                    key.clear()
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
                /* 
                From what I can gather there are two types of special localisation invocations:
                - $key$ - use that key instead of the key that was used to look up the string
                - [function(arg).function(arg)...] handling this one is going to be a nightmare
                */
                hmap = Some(data);
            }
        }
        //println!("{:?}", hmap);
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
            let d = data.get(key).unwrap().clone();
            if d.starts_with("$") && d.ends_with("$"){ //if we are provided with a different key to use instead then just do it
                return self.localize(&d[1..d.len()-1]);
            } else {
                //if the string contains []
                if d.contains('[') && d.contains(']'){ //handle the special function syntax
                    let mut value = d.to_string();
                    let mut start = 0;
                    let mut stack:Vec<(String, Vec<String>)> = Vec::new();
                    { //create a call stack
                        let mut call = String::new();
                        let mut args:Vec<String> = Vec::new();
                        let mut arg = String::new();
                        let mut collect = false;
                        let mut collect_args = false;
                        let mut ind:usize = 1;
                        for c in d.chars(){
                            match c {
                                '[' => {
                                    collect = true;
                                    start = ind - 1;
                                },
                                ']' => {
                                    collect = false;
                                    handle_stack(mem::take(&mut stack), start, &mut ind, &mut value)
                                },
                                '(' => {
                                    collect_args = true;
                                },
                                ')' => {
                                    collect_args = false;
                                    args.push(mem::take(&mut arg));
                                },
                                ',' => {
                                    args.push(mem::take(&mut arg));
                                },
                                '.' => {
                                    stack.push((mem::take(&mut call), mem::take(&mut args)));
                                },
                                _ => {
                                    if collect_args {
                                        arg.push(c);
                                    } else if collect {
                                        call.push(c);
                                    }
                                }
                            }
                            ind += 1;
                        }
                    }
                    return GameString::wrap(value);
                }
                return d;
            }
        }
        //println!("Key not found: {}", key);
        GameString::wrap(demangle_generic(key))
    }
}
