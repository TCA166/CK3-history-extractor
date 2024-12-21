use std::path::{Path, PathBuf};
use std::{fs, mem};

use super::super::{
    parser::GameString,
    types::{HashMap, Wrapper},
};

/// A function that demangles a generic name.
/// It will replace underscores with spaces and capitalize the first letter.
fn demangle_generic(input: &str) -> String {
    let mut s = input
        .trim_start_matches("dynn_")
        .trim_start_matches("nick_")
        .trim_start_matches("death_")
        .trim_start_matches("tenet_")
        .trim_start_matches("doctrine_")
        .trim_start_matches("ethos_")
        .trim_start_matches("heritage_")
        .trim_start_matches("language_")
        .trim_start_matches("martial_custom_")
        .trim_start_matches("tradition_")
        .trim_start_matches("e_")
        .trim_start_matches("k_")
        .trim_start_matches("d_")
        .trim_start_matches("c_")
        .trim_start_matches("b_")
        .trim_start_matches("x_x_")
        .trim_end_matches("_name")
        .trim_end_matches("_perk");
    let mut input_chars = s.chars();
    if input_chars.nth(1) == Some('p') && input_chars.nth(3) == Some('_') {
        s = s.split_at(4).1;
    }
    let mut s = s.replace("_", " ");
    if s.is_empty() {
        return s;
    }
    let bytes = unsafe { s.as_bytes_mut() };
    bytes[0] = bytes[0].to_ascii_uppercase();
    s
}

/// A function that handles the stack of function calls.
/// It will replace characters from start to end in result according to the functions and arguments in the stack.
#[inline(always)]
fn handle_stack(
    stack: Vec<(String, Vec<String>)>,
    start: usize,
    end: &mut usize,
    result: &mut String,
) {
    //TODO add more handling, will improve the accuracy of localization, especially for memories
    //println!("{:?}", stack);
    match stack.len() {
        2 => {
            if stack[0].0 == "GetTrait" && stack[1].0 == "GetName" {
                let l = stack[0].1[0].chars().count();
                let replace = demangle_generic(stack[0].1[0].as_str().trim_matches('\''));
                result.replace_range(start..*end, &replace);
                // move end to the end of the string
                *end = start + l;
            }
        }
        _ => {
            let replace: &str;
            if stack.len() > 0 && stack[0].1.len() > 0 {
                replace = stack[0].1[0].as_str();
            } else {
                replace = "";
            }
            result.replace_range(start..*end, replace);
            *end = start;
        }
    } // TODO add a catch for the CHARACTER.Custom('FR_E') and other stuff
      // MAYBE json input for easy customisation?
}

/// A function that resolves the special localisation invocations.
fn resolve_stack(str: &GameString) -> GameString {
    let mut value = str.to_string();
    let mut start = 0;
    let mut stack: Vec<(String, Vec<String>)> = Vec::new();
    {
        //create a call stack
        let mut call = String::new();
        let mut args: Vec<String> = Vec::new();
        let mut arg = String::new();
        let mut collect = false;
        let mut collect_args = false;
        let mut ind: usize = 1;
        for c in str.chars() {
            match c {
                '[' => {
                    collect = true;
                    start = ind - 1;
                }
                ']' => {
                    collect = false;
                    collect_args = false;
                    if !call.is_empty() {
                        stack.push((mem::take(&mut call), mem::take(&mut args)));
                    }
                    handle_stack(mem::take(&mut stack), start, &mut ind, &mut value)
                }
                '(' => {
                    if collect {
                        collect_args = true;
                    }
                }
                ')' => {
                    if collect_args {
                        collect_args = false;
                        if !arg.is_empty() {
                            args.push(mem::take(&mut arg));
                        }
                    }
                }
                ',' => {
                    if collect_args {
                        args.push(mem::take(&mut arg));
                    }
                }
                '.' => {
                    if collect {
                        if collect_args {
                            arg.push(c);
                        } else {
                            stack.push((mem::take(&mut call), mem::take(&mut args)));
                        }
                    }
                }
                _ => {
                    if collect_args {
                        arg.push(c);
                    } else if collect {
                        call.push(c);
                    }
                }
            }
            ind += c.len_utf8();
        }
    }
    return GameString::wrap(value);
}

// TODO add common phrase localization

/// An object that localizes strings.
/// It reads localization data from a directory and provides localized strings.
/// If the localization data is not found, it will demangle the key using an algorithm that tries to approximate the intended text
pub struct Localizer {
    data: HashMap<String, GameString>,
}

impl Localizer {
    /// Creates a new [Localizer] object.
    /// The object is empty and needs to be filled with localization data.
    /// After the data is added, the [Localizer::resolve] function should be called to resolve the special localisation invocations.
    pub fn new() -> Self {
        Localizer {
            data: HashMap::default(),
        }
    }

    /// Adds localization data from a directory.
    /// The path may be invalid, in which case the function will simply do nothing
    pub fn add_from_path(&mut self, localization_src_path: String) {
        // get every file in the directory and subdirectories
        let path = Path::new(&localization_src_path);
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
                                } else if entry.file_name().to_str().unwrap().ends_with(".yml") {
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
                for char in contents.chars() {
                    match char {
                        ' ' | '\t' => {
                            if quotes {
                                value.push(char);
                            }
                        }
                        '\n' => {
                            if past && !quotes && !value.is_empty() {
                                //Removing trait_? good idea because the localisation isnt consistent enough with trait names
                                //Removing _name though... controversial. Possibly a bad idea
                                //MAYBE only do this in certain files, but how to determine which are important? Pdx can change the format at any time
                                key = key
                                    .trim_start_matches("trait_")
                                    .trim_end_matches("_name")
                                    .to_string();
                                self.data.insert(
                                    mem::take(&mut key),
                                    GameString::wrap(mem::take(&mut value)),
                                );
                            } else {
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
        }
    }

    /// Resolves the special localisation invocations.
    pub fn resolve(&mut self) {
        /*
        From what I can gather there are two types of special localisation invocations:
        - $key$ - use that key instead of the key that was used to look up the string
        - [function(arg).function(arg)...] handling this one is going to be a nightmare
        */
        for (key, value) in self.data.clone() {
            // unfortunate clone, but we need to iterate over the data while modifying it
            // resolve the borrowed keys
            let mut new_value = String::new();
            let mut foreign_key = String::new();
            let mut in_key = false;
            for c in value.chars() {
                if c == '$' {
                    if in_key {
                        if let Some(localized) = self.data.get(&mem::take(&mut foreign_key)) {
                            new_value.push_str(localized.as_str());
                        }
                    }
                    in_key = !in_key;
                } else {
                    if in_key {
                        foreign_key.push(c);
                    } else {
                        new_value.push(c);
                    }
                }
            }
            self.data.insert(key.clone(), GameString::wrap(new_value));
        }
    }

    /// Localizes a string.
    pub fn localize(&mut self, key: &str) -> GameString {
        if let Some(d) = self.data.get(key) {
            //if the string contains []
            if d.contains('[') && d.contains(']') {
                //handle the special function syntax
                return resolve_stack(d);
            } else {
                return d.clone();
            }
        } else {
            let res = GameString::wrap(demangle_generic(key));
            self.data.insert(key.to_string(), res.clone());
            return res;
        }
    }
}

/// A trait that allows an object to be localized.
pub trait Localizable {
    /// Localizes the object.
    fn localize(&mut self, localization: &mut Localizer);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_links() {
        let mut localizer = Localizer::new();
        localizer
            .data
            .insert("key".to_string(), GameString::wrap("value".to_owned()));
        localizer
            .data
            .insert("test".to_string(), GameString::wrap("$key$".to_owned()));
        localizer
            .data
            .insert("test2".to_string(), GameString::wrap(" $key$ ".to_owned()));
        localizer.data.insert(
            "test3".to_string(),
            GameString::wrap(" $key$ $key$ ".to_owned()),
        );
        localizer.resolve();
        assert_eq!(localizer.data.get("test").unwrap().as_str(), "value");
        assert_eq!(localizer.data.get("test2").unwrap().as_str(), " value ");
        assert_eq!(
            localizer.data.get("test3").unwrap().as_str(),
            " value value "
        );
    }

    #[test]
    fn test_stack() {
        let mut localizer = Localizer::new();
        localizer.data.insert(
            "test".to_string(),
            GameString::wrap("[GetTrait(trait_test).GetName()]".to_owned()),
        );
        localizer.data.insert(
            "test2".to_string(),
            GameString::wrap("   [GetTrait(trait_test).GetName()]  ".to_owned()),
        );
        localizer.data.insert(
            "test3".to_string(),
            GameString::wrap(" hello( [GetTrait(trait_test).GetName()] ) ".to_owned()),
        );
        localizer.data.insert(
            "test4".to_string(),
            GameString::wrap(" hello,.(., [GetTrait(trait_test).GetName()] ) ".to_owned()),
        );
        localizer.resolve();
        assert_eq!(localizer.localize("test").as_str(), "Trait test");
        assert_eq!(localizer.localize("test2").as_str(), "   Trait test  ");
        assert_eq!(
            localizer.localize("test3").as_str(),
            " hello( Trait test ) "
        );
        assert_eq!(
            localizer.localize("test4").as_str(),
            " hello,.(., Trait test ) "
        );
    }

    #[test]
    fn test_handle_stack() {
        let mut stack: Vec<(String, Vec<String>)> = Vec::new();
        stack.push(("GetTrait".to_owned(), vec!["trait_test".to_owned()]));
        stack.push(("GetName".to_owned(), vec![]));
        let mut result = "trait_test".to_owned();
        let start = 0;
        let mut end = 10;
        handle_stack(stack, start, &mut end, &mut result);
        assert_eq!(result, "Trait test");
    }

    #[test]
    fn test_really_nasty() {
        let input = "[GetTrait(trait_test).GetName()]";
        let result = resolve_stack(&GameString::wrap(input.to_owned()));
        assert_eq!(result.as_str(), "Trait test");
    }

    #[test]
    fn test_french() {
        let input =
            "a brûlé [Select_CString(CHARACTER.IsFemale,'vive','vif')] dans un feu de forêt";
        resolve_stack(&GameString::wrap(input.to_owned()));
    }
}
