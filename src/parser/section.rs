use super::{
    super::types::{Shared, Wrapper, WrapperMut},
    game_object::{GameObjectArray, GameObjectMap, GameString, SaveFileObject, SaveFileValue},
};

use std::{fmt::Debug, mem, rc::Rc};

/// An error that can occur when parsing a section.
pub enum SectionError {
    InvalidSection(String),
    ParseError(&'static str),
}

impl Debug for SectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SectionError::InvalidSection(s) => write!(f, "Invalid section: {}", s),
            SectionError::ParseError(s) => write!(f, "Parse error: {}", s),
        }
    }
}

/// A stack
struct Stack {
    stack: Vec<SaveFileObject>,
    name: Option<String>,
}

impl Stack {
    fn new() -> Self {
        Stack {
            stack: Vec::new(),
            name: None,
        }
    }

    /// Push a new object named `key` to the stack.
    fn push(&mut self, key: &mut String) {
        if self.name.is_some() {
            if key.is_empty() {
                self.stack
                    .push(SaveFileObject::Array(GameObjectArray::from_name(
                        self.name.take().unwrap(),
                    )));
            } else {
                self.stack
                    .push(SaveFileObject::Map(GameObjectMap::from_name(
                        self.name.take().unwrap(),
                    )));
            }
        }
        // here's the thing, at this point we do not know if we are in an array or a map
        // we will know that when we reach the first token
        self.name = Some(mem::take(key));
    }

    /// Pop the current object from the stack and insert it into the parent object.
    fn pop(&mut self) {
        let val = self.stack.pop().unwrap();
        let parent = self.stack.last_mut().unwrap();
        match parent {
            SaveFileObject::Map(map) => {
                map.insert(val.get_name().to_owned(), SaveFileValue::Object(val));
            }
            SaveFileObject::Array(array) => {
                array.push(SaveFileValue::Object(val));
            }
        }
    }

    /// Insert a key value pair into the current object.
    fn insert(&mut self, key: &mut String, val: &mut String) -> Result<(), SectionError> {
        if self.name.is_some() {
            self.stack
                .push(SaveFileObject::Map(GameObjectMap::from_name(
                    self.name.take().unwrap(),
                )));
        } else if self.stack.is_empty() {
            key.clear();
            val.clear();
            return Ok(());
        }
        let val = SaveFileValue::String(GameString::wrap(mem::take(val)));
        match self.stack.last_mut().unwrap() {
            SaveFileObject::Map(map) => {
                map.insert(mem::take(key), val);
            }
            SaveFileObject::Array(array) => {
                let index = match key.parse::<usize>() {
                    Ok(val) => val,
                    Err(_) => {
                        return Err(SectionError::ParseError("Invalid index"));
                    }
                };
                key.clear();
                if index <= array.len() {
                    array.insert(index, val);
                } else {
                    // TODO technically this discards order, but I have yet to find a case where this is a problem
                    array.push(val);
                }
            }
        }
        Ok(())
    }

    /// Append a value to the current object.
    fn append(&mut self, key: &mut String) -> Result<(), SectionError> {
        if self.name.is_some() {
            self.stack
                .push(SaveFileObject::Array(GameObjectArray::from_name(
                    self.name.take().unwrap(),
                )));
        } else if self.stack.is_empty() {
            key.clear();
            return Ok(());
        }
        if key.is_empty() {
            return Ok(());
        }
        match self.stack.last_mut().unwrap() {
            SaveFileObject::Array(array) => {
                array.push(SaveFileValue::String(GameString::wrap(mem::take(key))));
            }
            _ => {
                return Err(SectionError::ParseError("Cannot append to a map"));
            }
        }
        Ok(())
    }

    /// Get the last object from the stack.
    fn get_last(&mut self) -> Option<SaveFileObject> {
        self.stack.pop()
    }
}

/// A struct that represents a section in a ck3 save file.
/// Each section has a name, holds a reference to the contents of the save file and the current parsing offset.
/// This means that all sections of the save file share a single parsing state.
///
/// # Validity
///
/// A section is valid until it is converted into a [SaveFileObject] using [Section::parse] or skipped using [Section::skip].
/// After that, the section is invalidated and any further attempts to convert it will result in a panic.
/// This is done to prevent double parsing of the same section.
///
/// # Example
///
/// ```
/// let save = SaveFile::open("save.ck3");
/// let section = save.next();
/// let object = section.to_object().unwrap();
/// ```
pub struct Section {
    name: String,
    contents: Rc<String>,
    offset: Shared<usize>,
    valid: bool,
}

impl Section {
    /// Create a new section
    pub fn new(name: String, contents: Rc<String>, offset: Shared<usize>) -> Self {
        Section {
            name: name,
            contents: contents,
            offset: offset,
            valid: true,
        }
    }

    /// Get the name of the section
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Invalidate the section.
    /// Both [Section::parse] and [Section::skip] will panic if the section is invalid.
    fn invalidate(&mut self) {
        self.valid = false;
    }

    /// Parse the section into a [SaveFileObject].
    /// This is a rather costly process as it has to read the entire section contents and parse them.
    /// You can then make a choice if you want to parse the object or [Section::skip] it.
    ///
    /// # Returns
    ///
    /// The parsed object. This can be a [GameObjectMap] or a [GameObjectArray].
    /// Note that empty objects are parsed as [GameObjectArray].
    /// Checking is object is empty via [SaveFileObject::is_empty] is a good idea before assuming it is a [GameObjectMap].
    pub fn parse(&mut self) -> Result<SaveFileObject, SectionError> {
        if !self.valid {
            return Err(SectionError::InvalidSection(self.name.clone()));
        }
        self.invalidate();
        let mut quotes = false;
        // storage
        let mut key = String::new(); // the key of the key value pair
        let mut val = String::new(); // the value of the key value pair
        let mut past_eq = false; // we use this flag to determine if we are parsing a key or a value
        let mut comment = false;
        let mut maybe_array = false; // if this is true, that means we encountered key =value
        let mut escape = false; // escape character toggle
        let mut stack = Stack::new();
        let mut depth: u32 = 0; // how deep we are in the object tree
                                //initialize the object stack
        let mut off = self.offset.get_internal_mut();

        /// Add a character to the key or value
        fn add_normal(c: char, key: &mut String, val: &mut String, past_eq: &mut bool) {
            if *past_eq {
                val.push(c);
            } else {
                key.push(c);
            }
        }

        //initialize the key stack
        for (ind, c) in self.contents[*off..].char_indices() {
            match c {
                // we parse the byte
                '\r' => {} //what? BILL GATES! HE CAN'T KEEP GETTING AWAY WITH IT!!!!!!
                '{' => {
                    // we have a new object, we push a new object
                    if comment {
                        continue;
                    }
                    if quotes || escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        maybe_array = false;
                        depth += 1;
                        stack.push(&mut key);
                        past_eq = false;
                    }
                }
                '}' => {
                    // we have reached the end of an object
                    if comment {
                        continue;
                    }
                    if quotes || escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        maybe_array = false;
                        // if there was an assignment, we insert the key value pair
                        if past_eq && !val.is_empty() {
                            stack.insert(&mut key, &mut val)?;
                            past_eq = false;
                        }
                        // if there wasn't an assignment but we still gathered some data
                        else {
                            stack.append(&mut key)?;
                        }
                        // resolved the object, time to append it to the parent object
                        depth -= 1;
                        if depth > 0 {
                            stack.pop();
                        } else if depth == 0 {
                            // we have reached the end of the object we are parsing, we return the object
                            *off += ind + 1;
                            break;
                        } else {
                            // sanity check
                            return Err(SectionError::ParseError("Invalid depth"));
                        }
                    }
                }
                '"' => {
                    // TODO handle integers separately
                    // we have a quote, we toggle the quotes flag
                    if comment {
                        continue;
                    }
                    if escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        quotes = !quotes;
                    }
                }
                '\n' => {
                    // we have reached the end of a line, we check if we have a key value pair
                    if comment {
                        comment = false;
                        continue;
                    }
                    if quotes || escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        maybe_array = false;
                        if past_eq {
                            // we have a key value pair
                            past_eq = false; // we reset the past_eq flag
                            stack.insert(&mut key, &mut val)?;
                        } else if !key.is_empty() {
                            // we have just a key { \n key \n }
                            stack.append(&mut key)?;
                        }
                    }
                }
                ' ' | '\t' => {
                    //syntax sugar we ignore, most of the time, unless...
                    if comment {
                        continue;
                    }
                    if quotes || escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        // we are not in quotes, we check if we have a key value pair
                        // we are key=value <-here
                        if past_eq && !val.is_empty() {
                            if val == "rgb" || val == "hsv" {
                                //we are here color=rgb {}
                                val.clear();
                            } else {
                                // in case {something=else something=else} or { 1 0=1 1=2 2=3 }
                                stack.insert(&mut key, &mut val)?;
                                past_eq = false;
                            }
                        } else if !key.is_empty() && !past_eq {
                            // in case { something something something } OR key =value we want to preserve the spaces
                            maybe_array = true;
                        }
                    }
                }
                '=' => {
                    if comment {
                        continue;
                    }
                    maybe_array = false;
                    // if we have an assignment, we toggle adding from key to value
                    if quotes || escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        past_eq = true;
                    }
                }
                '#' => {
                    if quotes || escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        comment = true;
                    }
                }
                '\\' => {
                    if escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        escape = true;
                    }
                }
                _ => {
                    //the main content we append to the key or value
                    if comment {
                        continue;
                    }
                    if maybe_array {
                        //we have a toggle that says that the last character was a space and key is not empty
                        stack.append(&mut key)?;
                        maybe_array = false;
                    }
                    //we simply append the character to the key or value
                    add_normal(c, &mut key, &mut val, &mut past_eq);
                }
            }
        }
        match stack.get_last() {
            Some(val) => {
                return Ok(val);
            }
            None => {
                return Err(SectionError::ParseError("Empty section"));
            }
        }
    }

    /// Skip the current section.
    /// Adds the length of the section to the offset and returns.
    /// This is useful if you are not interested in the contents of the section.
    ///
    /// # Panics
    ///
    /// Panics if the section is invalid.
    pub fn skip(&mut self) {
        if !self.valid {
            panic!("Section {} is invalid", self.name);
        }
        self.invalidate();
        let mut depth = 0;
        let mut off = self.offset.get_internal_mut();
        for (ind, c) in self.contents[*off..].char_indices() {
            match c {
                '{' => {
                    depth += 1;
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        *off += ind + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
    }
}
