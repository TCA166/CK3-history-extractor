use std::{
    collections::{hash_map, HashMap},
    fmt::Debug,
    rc::Rc,
    slice,
};

use super::types::{RefOrRaw, Wrapper};

/// A type alias for a game object id.
pub type GameId = u32;

// implementing the Wrapper trait for GameId is overkill, the opaqueness is not needed as it's always going to be a numeric type

/// A type alias for a game string.
/// Roughly meant to represent a raw string from a save file, reference counted so that it exists once in memory.
pub type GameString = Rc<String>;

impl Wrapper<String> for GameString {
    fn wrap(t: String) -> Self {
        Rc::new(t)
    }

    fn get_internal(&self) -> RefOrRaw<String> {
        RefOrRaw::Raw(self.as_ref())
    }
}

/// A value that can be stored in a SaveFile and is held by a GameObject.
/// This is a wrapper around a String or a GameObject.
pub enum SaveFileValue {
    String(GameString),
    Object(GameObject),
}

impl SaveFileValue {
    /// Get the value as a string
    ///
    /// # Panics
    ///
    /// Panics if the value is not a string
    ///
    /// # Returns
    ///
    /// A reference to the string
    pub fn as_string(&self) -> GameString {
        match self {
            SaveFileValue::String(s) => s.clone(),
            _ => panic!("Invalid value"),
        }
    }

    pub fn as_id(&self) -> GameId {
        self.as_string().parse::<GameId>().unwrap()
    }

    /// Get the value as a GameObject reference
    ///
    /// # Returns
    ///
    /// A reference to the GameObject
    pub fn as_object(&self) -> Option<&GameObject> {
        match self {
            SaveFileValue::Object(o) => Some(o),
            _ => None,
        }
    }

    /// Get the value as a mutable GameObject reference
    ///
    /// # Returns
    ///
    /// A mutable reference to the GameObject
    fn as_object_mut(&mut self) -> Option<&mut GameObject> {
        match self {
            SaveFileValue::Object(o) => Some(o),
            _ => None,
        }
    }
}

impl Debug for SaveFileValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveFileValue::String(s) => write!(f, "\"{}\"", s.as_ref()),
            SaveFileValue::Object(o) => write!(f, "{:?}", o),
        }
    }
}

impl PartialEq for SaveFileValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            SaveFileValue::String(s) => {
                if let SaveFileValue::String(other_s) = other {
                    s == other_s
                } else {
                    false
                }
            }
            SaveFileValue::Object(o) => {
                if let SaveFileValue::Object(other_o) = other {
                    o == other_o
                } else {
                    false
                }
            }
        }
    }
}

impl Clone for SaveFileValue {
    fn clone(&self) -> Self {
        match self {
            SaveFileValue::String(s) => SaveFileValue::String(s.clone()),
            SaveFileValue::Object(o) => SaveFileValue::Object(o.clone()),
        }
    }
}

/// Representation of a save file object.
/// These are the main data structure used to store game data.
/// Each belongs to a section, but that is not stored here.
/// Acts like a named dictionary and array, may be either or both or neither.
/// Each has a name, which isn't unique.
/// Holds [SaveFileValue]s, which are either strings or other GameObjects.
#[derive(Clone)]
pub struct GameObject {
    inner: HashMap<String, SaveFileValue>,
    array: Vec<SaveFileValue>,
    name: String,
}

impl GameObject {
    /// Create a new GameObject from a name
    pub fn from_name(name: String) -> GameObject {
        GameObject {
            inner: HashMap::new(),
            name: name,
            array: Vec::new(),
        }
    }

    /// Create a new empty GameObject
    pub fn new() -> GameObject {
        GameObject {
            inner: HashMap::new(),
            name: String::new(),
            array: Vec::new(),
        }
    }

    /// Rename the GameObject
    pub fn rename(&mut self, name: String) {
        self.name = name;
    }

    /// Insert a new key value pair into the GameObject dictionary.
    /// Detects if the key has been inserted and in such a case converts the value held under the key into an array.
    pub fn insert(&mut self, key: String, value: SaveFileValue) {
        if self.inner.contains_key(&key) {
            let val = self.inner.get_mut(&key).unwrap();
            if val == &value {
                return;
            }
            let val_obj = val.as_object_mut();
            if val_obj.is_some() && val_obj.as_ref().unwrap().is_array() {
                let arr = val_obj.unwrap();
                arr.push(value);
            } else {
                let mut arr = GameObject::from_name(key.clone());
                arr.push(val.clone());
                arr.push(value);
                self.inner.insert(key, SaveFileValue::Object(arr));
            }
        } else {
            self.inner.insert(key, value);
        }
    }

    /// Get the value of a key
    pub fn get(&self, key: &str) -> Option<&SaveFileValue> {
        self.inner.get(key)
    }

    /// Get the value of a key as a string.
    /// Mainly used for convenience.
    ///
    /// # Panics
    ///
    /// If the key is missing or the value is not a string
    ///
    pub fn get_string_ref(&self, key: &str) -> GameString {
        self.inner.get(key).unwrap().as_string()
    }

    /// Get the value of a key as a GameObject.
    /// Mainly used for convenience.
    ///
    /// # Panics
    ///
    /// If the key is missing or the value is not a GameObject
    ///
    pub fn get_object_ref(&self, key: &str) -> &GameObject {
        self.inner.get(key).unwrap().as_object().unwrap()
    }

    /// Get the value of an index in the GameObject array
    #[allow(dead_code)]
    pub fn get_index(&self, index: usize) -> Option<&SaveFileValue> {
        self.array.get(index)
    }

    /// Get the name of the GameObject
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Push a new value into the GameObject array
    pub fn push(&mut self, value: SaveFileValue) {
        self.array.push(value);
    }

    /// Checks if the dictionary and array are empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty() && self.array.is_empty()
    }

    /// Gets the iterator for the underlying array
    pub fn get_array_iter(&self) -> slice::Iter<SaveFileValue> {
        self.array.iter()
    }

    /// Gets the iterator for the underlying dictionary
    pub fn get_obj_iter(&self) -> hash_map::Iter<String, SaveFileValue> {
        self.inner.iter()
    }

    /// Get the keys of the GameObject dictionary
    pub fn get_keys(&self) -> hash_map::Keys<String, SaveFileValue> {
        self.inner.keys()
    }

    pub fn is_array(&self) -> bool {
        self.inner.is_empty() && !self.array.is_empty()
    }
}

impl Debug for GameObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = write!(f, "{{name:{}", self.name);
        if r.is_err() {
            return r;
        }
        if !self.array.is_empty() {
            let r = write!(f, "{:?}", self.array);
            if r.is_err() {
                return r;
            }
        }
        if !self.inner.is_empty() {
            let r = write!(f, "{:?}", self.inner);
            if r.is_err() {
                return r;
            }
        }
        let r = write!(f, "}}");
        return r;
    }
}

impl PartialEq for GameObject {
    fn eq(&self, other: &Self) -> bool {
        let mut eq = self.array == other.array;
        eq = eq && self.inner == other.inner;
        eq = eq && self.name == other.name;
        eq
    }
}
