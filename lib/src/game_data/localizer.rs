use std::path::{Path, PathBuf};
use std::{fmt, fs, mem};

use super::super::types::{GameString, HashMap};

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

/// An object that localizes strings.
/// It reads localization data from a directory and provides localized strings.
/// If the localization data is not found, it will demangle the key using an algorithm that tries to approximate the intended text
pub struct Localizer {
    /// Whether at least a single file has been loaded
    initialized: bool,
    data: HashMap<String, GameString>,
}

impl Default for Localizer {
    fn default() -> Self {
        Localizer {
            initialized: false,
            data: HashMap::default(),
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
        self.initialized = true;
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
                        self.data
                            .insert(mem::take(&mut key), GameString::from(mem::take(&mut value)));
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
    - # #! - these are formatting instructions, can be nested
    */

    pub fn remove_formatting(&mut self) {
        for (_, value) in self.data.iter_mut() {
            let mut new = String::with_capacity(value.len());
            let mut iter = value.chars();
            let mut open = false;
            let mut func_open = false;
            while let Some(c) = iter.next() {
                match c {
                    '#' => {
                        if open {
                            open = false;
                            if let Some(next) = iter.next() {
                                // we skip the ! in #!
                                if next != '!' {
                                    new.push(next);
                                }
                            }
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
                    '$' => {
                        func_open = !func_open;
                        new.push(c);
                    }
                    '[' => {
                        func_open = true;
                        new.push(c);
                    }
                    ']' => {
                        func_open = false;
                        new.push(c);
                    }
                    '|' => {
                        if func_open {
                            while let Some(c) = iter.next() {
                                if c == ']' {
                                    new.push(c);
                                    break;
                                }
                            }
                        } else {
                            new.push(c);
                        }
                    }
                    _ => {
                        new.push(c);
                    }
                }
            }
            *value = GameString::from(new);
        }
    }
}

/// A localization query. A function name and a list of arguments.
pub type LocalizationQuery = (String, Vec<String>);

/// A stack of localization queries.
pub type LocalizationStack = Vec<LocalizationQuery>;

/// An error that occurs when localizing a string.
#[derive(Debug)]
pub enum LocalizationError {
    InvalidQuery(GameString, LocalizationStack),
    LocalizationSyntaxError(&'static str),
}

fn create_localization_stack(input: String) -> Result<LocalizationStack, LocalizationError> {
    // MAYBE in future resolve recursively the arguments? as of right now theoretically the arguments may themselves be functions and we don't handle that
    let mut stack: LocalizationStack = Vec::new();
    let mut call = String::new();
    let mut args: Vec<String> = Vec::new();
    let mut arg = String::new();
    let mut collect_args = false;
    for char in input.chars() {
        match char {
            '(' => {
                collect_args = true;
            }
            ')' => {
                collect_args = false;
                if !arg.is_empty() {
                    args.push(mem::take(&mut arg));
                }
            }
            ',' => {
                if collect_args {
                    args.push(mem::take(&mut arg));
                }
            }
            '.' => {
                if collect_args {
                    arg.push(char);
                } else {
                    stack.push((mem::take(&mut call), mem::take(&mut args)));
                }
            }
            ']' => {
                Err(LocalizationError::LocalizationSyntaxError(
                    "unexpected ']' character",
                ))?;
            }
            '\'' => {} // ignore
            _ => {
                if collect_args {
                    arg.push(char);
                } else {
                    call.push(char);
                }
            }
        }
    }
    stack.push((call, args));
    Ok(stack)
}

impl fmt::Display for LocalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalizationError::InvalidQuery(val, stack) => {
                write!(f, "a query: {:?} in {} is in some way invalid.", stack, val)
            }
            LocalizationError::LocalizationSyntaxError(s) => {
                write!(f, "localization syntax error: {}", s)
            }
        }
    }
}

impl std::error::Error for LocalizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// An object that can be used to localize strings
pub trait Localize<T: AsRef<str> + From<String>> {
    /// A simple function that looks up raw value associated with the given localization key
    fn lookup<K: AsRef<str>>(&self, key: K) -> Option<T>;

    fn is_empty(&self) -> bool;

    /// A simple localization function that will return the localized string.
    /// It assumes that the key is not complex and does not require any special handling.
    fn localize<K: AsRef<str>>(&self, key: K) -> Result<T, LocalizationError> {
        self.localize_query(key, |_| -> Option<&str> { None })
    }

    /// A localization function that will return the localized string.
    /// It assumes a more complex key, resolving $provider$ into the value.
    /// More complex keys will not be resolved.
    fn localize_provider<K: AsRef<str>>(
        &self,
        key: K,
        provider: &str,
        value: &str,
    ) -> Result<T, LocalizationError> {
        let query = |q: &LocalizationStack| {
            if q.len() == 1 && q.first().unwrap().0 == provider {
                Some(value)
            } else {
                None
            }
        };
        self.localize_query(key, query)
    }

    /// A localization function that will return the localized string.
    /// It allows for complete control over the complex key resolution.
    /// Every time a $key$ or [function(arg)] is encountered, the query function will be called.
    /// The query function should return the value in accordance to the provided stack, or None if the value is not found.
    /// Whether None causes an error or not is up to the implementation.
    fn localize_query<K: AsRef<str>, S: AsRef<str>, F: Fn(&LocalizationStack) -> Option<S>>(
        &self,
        key: K,
        query: F,
    ) -> Result<T, LocalizationError> {
        if let Some(d) = self.lookup(key.as_ref()) {
            let value = d.as_ref();
            // we have A template localization string, now we have to resolve it
            let mut collect = false;
            let mut collection = String::with_capacity(value.len());
            let mut arg = String::new();
            // this is technically a less efficient way of doing it, but it's easier to read
            for c in value.chars() {
                match c {
                    '$' => {
                        collect = !collect;
                        if !collect {
                            if let Some(val) = self.lookup(&arg) {
                                collection.push_str(val.as_ref());
                                arg.clear();
                            } else {
                                let stack = vec![(mem::take(&mut arg), Vec::new())];
                                if let Some(val) = query(&stack) {
                                    collection.push_str(val.as_ref());
                                } else {
                                    if cfg!(feature = "permissive") {
                                        collection
                                            .push_str(demangle_generic(arg.as_ref()).as_str());
                                    } else {
                                        return Err(LocalizationError::InvalidQuery(
                                            value.into(),
                                            stack,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    '[' => {
                        if collect {
                            return Err(LocalizationError::LocalizationSyntaxError(
                                "unexpected '[' character",
                            ));
                        } else {
                            collect = true;
                        }
                    }
                    ']' => {
                        if collect {
                            collect = false;
                            let stack = create_localization_stack(mem::take(&mut arg))?;
                            if let Some(val) = query(&stack) {
                                collection.push_str(val.as_ref());
                            } else {
                                if !cfg!(feature = "permissive") {
                                    return Err(LocalizationError::InvalidQuery(
                                        value.into(),
                                        stack,
                                    ));
                                }
                            }
                        } else {
                            return Err(LocalizationError::LocalizationSyntaxError(
                                "unexpected ']' character",
                            ));
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
            return Ok(collection.into());
        } else {
            if !cfg!(feature = "permissive")
                && !self.is_empty()
                && !key.as_ref().is_empty()
                && key.as_ref().contains('_')
            {
                eprintln!("Warning: key {} not found", key.as_ref());
            }
            return Ok(demangle_generic(key.as_ref()).into());
        }
    }
}

impl Localize<GameString> for Localizer {
    fn lookup<K: AsRef<str>>(&self, key: K) -> Option<GameString> {
        self.data.get(key.as_ref()).cloned()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(feature = "serde")]
mod serialize {
    use super::Localizer;
    use serde::Serialize;

    impl Serialize for Localizer {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.data.serialize(serializer)
        }
    }
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
            .insert("key".to_string(), GameString::from("value"));
        localizer
            .data
            .insert("test".to_string(), GameString::from("$key$"));
        localizer
            .data
            .insert("test2".to_string(), GameString::from(" $key$ "));
        localizer
            .data
            .insert("test3".to_string(), GameString::from(" $key$ $key$ "));
        assert_eq!(localizer.localize("key").unwrap().as_ref(), "value");
        assert_eq!(localizer.localize("test").unwrap().as_ref(), "value");
        assert_eq!(localizer.localize("test2").unwrap().as_ref(), " value ");
        assert_eq!(
            localizer.localize("test3").unwrap().as_ref(),
            " value value "
        );
    }

    #[test]
    fn test_remove_formatting() {
        let mut localizer = Localizer::default();
        localizer
            .data
            .insert("test".to_string(), GameString::from("#P value#! # #!"));
        localizer
            .data
            .insert("test2".to_string(), GameString::from("[test|U] [test|idk]"));
        localizer.remove_formatting();
        assert_eq!(localizer.localize("test").unwrap().as_ref(), "value ");
        assert_eq!(
            localizer.data.get("test2").unwrap().as_ref(),
            "[test] [test]"
        );
    }

    #[test]
    fn test_stack() {
        let mut localizer = Localizer::default();
        localizer
            .data
            .insert("trait_test".to_string(), GameString::from("Trait test"));
        localizer.data.insert(
            "test".to_string(),
            GameString::from("[GetTrait(trait_test).GetName()]"),
        );
        localizer.data.insert(
            "test2".to_string(),
            GameString::from("   [GetTrait(trait_test).GetName()]  "),
        );
        localizer.data.insert(
            "test3".to_string(),
            GameString::from(" hello( [GetTrait(trait_test).GetName()] ) "),
        );
        localizer.data.insert(
            "test4".to_string(),
            GameString::from(" hello,.(., [GetTrait(trait_test).GetName()] ) "),
        );
        let query = |stack: &LocalizationStack| Some(localizer.localize(&stack[0].1[0]).unwrap());
        assert_eq!(
            localizer.localize_query("test", query).unwrap().as_ref(),
            "Trait test"
        );
        assert_eq!(
            localizer.localize_query("test2", query).unwrap().as_ref(),
            "   Trait test  "
        );
        assert_eq!(
            localizer.localize_query("test3", query).unwrap().as_ref(),
            " hello( Trait test ) "
        );
        assert_eq!(
            localizer.localize_query("test4", query).unwrap().as_ref(),
            " hello,.(., Trait test ) "
        );
    }

    #[test]
    fn test_really_nasty() {
        let result =
            create_localization_stack("GetTrait(trait_test).GetName()".to_owned()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "GetTrait");
        assert_eq!(result[0].1.len(), 1);
        assert_eq!(result[0].1[0], "trait_test");
        assert_eq!(result[1].0, "GetName");
        assert_eq!(result[1].1.len(), 0);
    }

    #[test]
    fn test_french() {
        let input = "Select_CString(CHARACTER.IsFemale,'brûlé','vif')";
        let result = create_localization_stack(input.to_owned()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "Select_CString");
        assert_eq!(result[0].1.len(), 3);
        assert_eq!(result[0].1[0], "CHARACTER.IsFemale");
        assert_eq!(result[0].1[1], "brûlé");
        assert_eq!(result[0].1[2], "vif");
    }
}
