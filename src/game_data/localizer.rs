use std::path::{Path, PathBuf};
use std::{fs, mem};

use super::super::{
    parser::GameString,
    types::{HashMap, Wrapper},
};

/* This is an imperfect localization parser. Unfortunately, the localization
files are far too complex to be parsed without also implementing a whole
game around it. This is a simple parser that will handle the most common
cases WE will encounter.
https://ck3.paradoxwikis.com/Localization - very important page.
we do want to handle $$ syntax, and [] function args, but the formatting? idk probably not
*/

/// A function that demangles a generic name.
/// It will replace underscores with spaces and capitalize the first letter.
fn demangle_generic(input: &str) -> String {
    const PREFIXES: [&str; 16] = [
        "dynn_",
        "nick_",
        "death_",
        "tenet_",
        "doctrine_",
        "ethos_",
        "heritage_",
        "language_",
        "martial_custom_",
        "tradition_",
        "e_",
        "k_",
        "d_",
        "c_",
        "b_",
        "x_x_",
    ];
    const SUFFIXES: [&str; 2] = ["_name", "_perk"];

    let mut s = input;
    for prefix in PREFIXES {
        if let Some(stripped) = s.strip_prefix(prefix) {
            s = stripped;
            break;
        }
    }
    for suffix in SUFFIXES {
        if let Some(stripped) = s.strip_suffix(suffix) {
            s = stripped;
            break;
        }
    }
    let mut s = s.replace("_", " ");
    if s.is_empty() {
        return s;
    }
    let first = s.chars().nth(0).unwrap();
    if first.is_ascii_alphabetic() {
        s[0..1].make_ascii_uppercase();
    }
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
      // MAYBE json input for easy customisation? dump_data_types can dump game types. Is that useful info?
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

/// An object that localizes strings.
/// It reads localization data from a directory and provides localized strings.
/// After the data is added, the [Localizer::resolve] function should be called to resolve the special localization invocations.
/// If the localization data is not found, it will demangle the key using an algorithm that tries to approximate the intended text
pub struct Localizer {
    data: HashMap<String, GameString>,
}

impl Default for Localizer {
    fn default() -> Self {
        Localizer {
            data: HashMap::new(),
        }
    }
}

impl Localizer {
    /// Adds localization data from a directory.
    /// The path may be invalid, in which case the function will simply do nothing
    pub fn add_from_path<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        if path.is_dir() {
            // a stack to keep track of the directories
            let mut stack: Vec<PathBuf> = vec![PathBuf::from(path)];
            // a vector to keep track of all the files
            let mut all_files: Vec<PathBuf> = Vec::new();
            while let Some(entry) = stack.pop() {
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
                // add the file to the localizer
                self.add_localization_file(&contents);
            }
        }
    }

    pub fn add_localization_file(&mut self, contents: &str) {
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
                        // MAYBE change this?
                        //Removing trait_? good idea because the localization isn't consistent enough with trait names
                        //Removing _name though... controversial. Possibly a bad idea
                        key = key
                            .trim_start_matches("trait_")
                            .trim_end_matches("_name")
                            .to_string();
                        self.data
                            .insert(mem::take(&mut key), GameString::wrap(mem::take(&mut value)));
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

    /*
    From what I can gather there are three types of special localisation invocations:
    - $key$ - use that key instead of the key that was used to look up the string
    - [function(arg).function(arg)...] handling this one is going to be a nightmare
    - # #! - these are formatting instructions
    */

    pub fn remove_formatting(&mut self) {
        for (_, value) in self.data.iter_mut() {
            let mut new = String::with_capacity(value.len());
            let mut iter = value.chars();
            let mut open = false;
            while let Some(c) = iter.next() {
                match c {
                    '#' => {
                        if open {
                            open = false;
                            iter.next(); // we skip the ! in #!
                        } else {
                            open = true;
                            // skip to space
                            while let Some(c) = iter.next() {
                                if c == ' ' {
                                    break;
                                }
                            }
                        }
                    }
                    _ => {
                        new.push(c);
                    }
                }
            }
            *value = GameString::wrap(new);
        }
    }
}

pub trait Localize {
    fn localize(&mut self, key: &str) -> GameString {
        self.localize_query(key, |query| panic!("Unexpected query {}", query))
    }

    fn localize_value(&mut self, key: &str, value: &str) -> GameString {
        let query = |q: &str| {
            if q == "VALUE" {
                value.to_string()
            } else {
                panic!("Invalid query: {}", q);
            }
        };
        self.localize_query(key, query)
    }

    fn localize_query<F: Fn(&str) -> String>(&mut self, key: &str, query: F) -> GameString;
}

impl Localize for Localizer {
    // TODO change query arg type, will probably need to have a special general type like a stack of function name and arguments
    fn localize_query<F: Fn(&str) -> String>(&mut self, key: &str, query: F) -> GameString {
        if let Some(d) = self.data.get(key) {
            // we have A template localization string, now we have to resolve it
            let mut collect = false;
            let mut collection = String::with_capacity(d.len());
            let mut arg = String::new();
            for c in d.chars() {
                match c {
                    '$' => {
                        collect = !collect;
                        if !collect {
                            collection.push_str(&query(mem::take(&mut arg).as_str()).as_str());
                        }
                    }
                    '[' => {
                        if collect {
                            arg.push('[');
                        } else {
                            collect = true;
                        }
                    }
                    ']' => {
                        if collect {
                            collect = false;
                            collection.push_str(&query(mem::take(&mut arg).as_str()).as_str());
                        } else {
                            collection.push(']');
                        }
                    }
                    _ => {
                        if collect {
                            arg.push(c);
                        } else {
                            collection.push(c);
                        }
                    }
                }
            }
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
    fn localize<L: Localize>(&mut self, localization: &mut L);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demangle_generic() {
        assert_eq!(demangle_generic("dynn_test_name"), "Test");
        assert_eq!(demangle_generic("dynn_test_perk"), "Test");
        assert_eq!(demangle_generic("dynn_test"), "Test");
    }

    #[test]
    fn test_links() {
        let mut localizer = Localizer::default();
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
        assert_eq!(localizer.data.get("test").unwrap().as_str(), "value");
        assert_eq!(localizer.data.get("test2").unwrap().as_str(), " value ");
        assert_eq!(
            localizer.data.get("test3").unwrap().as_str(),
            " value value "
        );
    }

    #[test]
    fn test_stack() {
        let mut localizer = Localizer::default();
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
