use std::{
    any::type_name,
    error,
    fmt::{self, Debug, Display},
    num::{ParseFloatError, ParseIntError},
    ops::Index,
    rc::Rc,
    str::ParseBoolError,
};

use super::super::types::{HashMap, HashMapIter, RefOrRaw, Wrapper};

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

/// An error that can occur when converting a value from a save file.
#[derive(Debug)]
pub enum ConversionError {
    /// The value is not of the expected type.
    InvalidType(SaveFileValue, &'static str),
    /// The value is not a valid value.
    InvalidValue,
}

impl From<ParseIntError> for ConversionError {
    fn from(_: ParseIntError) -> Self {
        ConversionError::InvalidValue
    }
}

impl From<ParseFloatError> for ConversionError {
    fn from(_: ParseFloatError) -> Self {
        ConversionError::InvalidValue
    }
}

impl From<ParseBoolError> for ConversionError {
    fn from(_: ParseBoolError) -> Self {
        ConversionError::InvalidValue
    }
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidType(t1, t2) => write!(f, "failed converting {:?} to {}", t1, t2),
            Self::InvalidValue => write!(f, "the value is invalid for conversion"),
        }
    }
}

impl error::Error for ConversionError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

/// A value that comes from a save file.
/// Matching against this enum is a bad idea, because [SaveFileValue::String] may actually contain any type.
/// It's better to use the conversion methods like [SaveFileValue::as_string].
#[derive(PartialEq, Clone, Debug)]
pub enum SaveFileValue {
    /// A simple string value, may be anything in reality.
    String(GameString),
    /// A complex object value.
    Object(SaveFileObject),
    /// A floating point value
    Real(f64),
    /// An integer
    Integer(i64),
    /// A boolean
    Boolean(bool),
}

impl SaveFileValue {
    // this API allows for easy error collection using the ? operator.
    /// Get the value as a string
    pub fn as_string(&self) -> Result<GameString, ConversionError> {
        match self {
            SaveFileValue::String(s) => Ok(s.clone()),
            _ => Err(ConversionError::InvalidType(
                self.clone(),
                type_name::<GameString>(),
            )),
        }
    }

    /// Get the value as a GameId
    pub fn as_id(&self) -> Result<GameId, ConversionError> {
        return Ok(self.as_integer()? as GameId);
    }

    /// Get the value as a GameObject
    pub fn as_object(&self) -> Result<&SaveFileObject, ConversionError> {
        match self {
            SaveFileValue::Object(o) => Ok(o),
            _ => Err(ConversionError::InvalidType(
                self.clone(),
                type_name::<SaveFileObject>(),
            )),
        }
    }

    pub fn as_integer(&self) -> Result<i64, ConversionError> {
        match self {
            SaveFileValue::Integer(i) => Ok(*i),
            SaveFileValue::String(s) => Ok(s.parse::<i64>()?),
            _ => Err(ConversionError::InvalidType(
                self.clone(),
                type_name::<i64>(),
            )),
        }
    }

    pub fn as_real(&self) -> Result<f64, ConversionError> {
        match self {
            SaveFileValue::Real(r) => Ok(*r),
            SaveFileValue::String(s) => Ok(s.parse::<f64>()?),
            _ => Err(ConversionError::InvalidType(
                self.clone(),
                type_name::<f64>(),
            )),
        }
    }

    pub fn as_boolean(&self) -> Result<bool, ConversionError> {
        match self {
            SaveFileValue::Boolean(b) => Ok(*b),
            SaveFileValue::Integer(i) => Ok(*i != 0),
            SaveFileValue::String(s) => Ok(s.parse::<bool>()?),
            _ => Err(ConversionError::InvalidType(
                self.clone(),
                type_name::<bool>(),
            )),
        }
    }
}

/// A game object that stores values as a map.
pub type GameObjectMap = GameObject<HashMap<String, SaveFileValue>>;
/// A game object that stores values as an array.
pub type GameObjectArray = GameObject<Vec<SaveFileValue>>;

/// An object that comes from a save file.
#[derive(PartialEq, Clone)]
pub enum SaveFileObject {
    /// An object that stores values as a map.
    Map(GameObjectMap),
    /// An object that stores values as an array.
    Array(GameObjectArray),
}

impl SaveFileObject {
    /// Get the name of the object
    pub fn get_name(&self) -> &str {
        match self {
            SaveFileObject::Map(m) => m.get_name(),
            SaveFileObject::Array(a) => a.get_name(),
        }
    }

    /// Get the value as a GameObject map
    ///
    /// # Panics
    ///
    /// Panics if the value is not a map
    pub fn as_map(&self) -> Result<&GameObjectMap, ConversionError> {
        match self {
            SaveFileObject::Map(o) => Ok(o),
            _ => Err(ConversionError::InvalidType(
                SaveFileValue::Object(self.clone()),
                type_name::<GameObjectMap>(),
            )),
        }
    }

    /// Get the value as a mutable GameObject map
    ///
    /// # Panics
    ///
    /// Panics if the value is not a map
    pub fn as_map_mut(&mut self) -> Result<&mut GameObjectMap, ConversionError> {
        match self {
            SaveFileObject::Map(o) => Ok(o),
            _ => Err(ConversionError::InvalidType(
                SaveFileValue::Object(self.clone()),
                type_name::<GameObjectMap>(),
            )),
        }
    }

    /// Get the value as a GameObject array
    ///
    /// # Panics
    ///
    /// Panics if the value is not an array
    pub fn as_array(&self) -> Result<&GameObjectArray, ConversionError> {
        match self {
            SaveFileObject::Array(a) => Ok(a),
            _ => Err(ConversionError::InvalidType(
                SaveFileValue::Object(self.clone()),
                type_name::<GameObjectArray>(),
            )),
        }
    }

    /// Get the value as a mutable GameObject array
    ///
    /// # Panics
    ///
    /// Panics if the value is not an array
    pub fn as_array_mut(&mut self) -> Result<&mut GameObjectArray, ConversionError> {
        match self {
            SaveFileObject::Array(a) => Ok(a),
            _ => Err(ConversionError::InvalidType(
                SaveFileValue::Object(self.clone()),
                type_name::<GameObjectArray>(),
            )),
        }
    }

    /// Rename the object
    pub fn rename(&mut self, name: String) {
        match self {
            SaveFileObject::Map(m) => m.rename(name),
            SaveFileObject::Array(a) => a.rename(name),
        }
    }

    /// Check if the object is empty
    pub fn is_empty(&self) -> bool {
        match self {
            SaveFileObject::Map(m) => m.is_empty(),
            SaveFileObject::Array(a) => a.is_empty(),
        }
    }
}

impl Debug for SaveFileObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveFileObject::Map(o) => write!(f, "Map({},{:?})", o.name, o.inner),
            SaveFileObject::Array(o) => write!(f, "Array({},{:?})", o.name, o.inner),
        }
    }
}

/// A trait describing a collection that can be used as storage in [GameObject].
/// This is really just a way to abstract over [HashMap] and [Vec].
pub trait GameObjectCollection: Debug {
    /// Create a new instance of the collection
    fn new() -> Self;
    /// Check if the collection is empty
    fn is_empty(&self) -> bool;
}

impl GameObjectCollection for HashMap<String, SaveFileValue> {
    fn new() -> Self {
        HashMap::default()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<T: Debug> GameObjectCollection for Vec<T> {
    fn new() -> Self {
        Vec::new()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

/// Representation of a save file object.
/// These are the main data structure used to store game data.
/// Each belongs to a section, but that is not stored here.
/// Each has a name, which isn't unique.
/// Holds [SaveFileValue]s, which are either strings or other GameObjects.
#[derive(PartialEq, Clone)]
pub struct GameObject<T: GameObjectCollection> {
    inner: T,
    name: String,
}

impl<T: GameObjectCollection> GameObject<T> {
    /// Create a new GameObject from a name
    pub fn from_name(name: String) -> Self {
        GameObject {
            inner: T::new(),
            name: name,
        }
    }

    /// Create a new empty GameObject
    pub fn new() -> Self {
        GameObject {
            inner: T::new(),
            name: String::new(),
        }
    }

    /// Rename the GameObject
    pub fn rename(&mut self, name: String) {
        self.name = name;
    }

    /// Get the name of the GameObject
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Check if the GameObject is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[derive(Debug)]
pub enum SaveObjectError {
    ConversionError(ConversionError),
    KeyError(KeyError),
}

impl fmt::Display for SaveObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConversionError(e) => write!(f, "conversion error: {}", e),
            Self::KeyError(e) => write!(f, "key error: {}", e),
        }
    }
}

impl error::Error for SaveObjectError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::ConversionError(e) => Some(e),
            Self::KeyError(e) => Some(e),
        }
    }
}

impl From<ConversionError> for SaveObjectError {
    fn from(e: ConversionError) -> Self {
        SaveObjectError::ConversionError(e)
    }
}

impl From<KeyError> for SaveObjectError {
    fn from(e: KeyError) -> Self {
        SaveObjectError::KeyError(e)
    }
}

#[derive(Debug)]
pub enum KeyError {
    MissingKey(String, GameObjectMap),
    IndexError(usize, GameObjectArray),
    ArrEmpty,
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey(key, obj) => write!(f, "key {} missing from object {:?}", key, obj),
            Self::IndexError(index, obj) => write!(f, "index {} out of range for {:?}", index, obj),
            Self::ArrEmpty => write!(f, "array is empty"),
        }
    }
}

impl error::Error for KeyError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl GameObject<HashMap<String, SaveFileValue>> {
    /// Get the value of a key as a mutable GameObject.
    /// This is the lower level interface to the object that allows for more complex operations.
    pub fn get(&self, key: &str) -> Option<&SaveFileValue> {
        self.inner.get(key)
    }

    /// A getter that returns a result
    fn get_err(&self, key: &str) -> Result<&SaveFileValue, KeyError> {
        self.get(key) // lazy error initialization, else we copy key and obj every time
            .ok_or_else(|| KeyError::MissingKey(key.to_owned(), self.clone()))
    }

    /// Get the value of a key as a string.
    pub fn get_string(&self, key: &str) -> Result<GameString, SaveObjectError> {
        Ok(self.get_err(key)?.as_string()?)
    }

    /// Get the value of a key as a boolean.
    pub fn get_object(&self, key: &str) -> Result<&SaveFileObject, SaveObjectError> {
        Ok(self.get_err(key)?.as_object()?)
    }

    /// Get the value of a key as an integer.
    pub fn get_integer(&self, key: &str) -> Result<i64, SaveObjectError> {
        Ok(self.get_err(key)?.as_integer()?)
    }

    /// Get the value of a key as a real number.
    pub fn get_real(&self, key: &str) -> Result<f64, SaveObjectError> {
        Ok(self.get_err(key)?.as_real()?)
    }

    /// Get the value of a key as a GameId.
    pub fn get_game_id(&self, key: &str) -> Result<GameId, SaveObjectError> {
        Ok(self.get_err(key)?.as_id()?)
    }
    /// Insert a new value into the object.
    /// If the key already exists, the value at that key alongside the new value will be stored in an array at that key.
    /// Thus held values are never discarded and here the multi key feature of the save file format is implemented.
    pub fn insert(&mut self, key: String, value: SaveFileValue) {
        let stored = self.inner.get_mut(&key);
        match stored {
            Some(val) => match val {
                SaveFileValue::Object(SaveFileObject::Array(arr)) => {
                    arr.push(value);
                }
                _ => {
                    let mut arr = GameObjectArray::from_name(key.clone());
                    arr.push(val.clone());
                    arr.push(value);
                    self.inner
                        .insert(key, SaveFileValue::Object(SaveFileObject::Array(arr)));
                }
            },
            None => {
                self.inner.insert(key, value);
            }
        }
    }
}

impl<'a> IntoIterator for &'a GameObject<HashMap<String, SaveFileValue>> {
    type Item = (&'a String, &'a SaveFileValue);
    type IntoIter = HashMapIter<'a, String, SaveFileValue>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.inner).into_iter()
    }
}

impl GameObject<Vec<SaveFileValue>> {
    /// Get the value at an index
    pub fn get_index(&self, index: usize) -> Result<&SaveFileValue, KeyError> {
        self.inner
            .get(index)
            .ok_or(KeyError::IndexError(index, self.clone()))
    }

    /// Get the length of the array
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Push a value to the array
    pub fn push(&mut self, value: SaveFileValue) {
        self.inner.push(value);
    }

    /// Insert a value at an index
    pub fn insert(&mut self, index: usize, value: SaveFileValue) {
        self.inner.insert(index, value);
    }

    pub fn pop(&mut self) -> Result<SaveFileValue, KeyError> {
        self.inner.pop().ok_or(KeyError::ArrEmpty)
    }
}

impl Index<usize> for GameObject<Vec<SaveFileValue>> {
    type Output = SaveFileValue;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<'a> IntoIterator for &'a GameObject<Vec<SaveFileValue>> {
    type Item = &'a SaveFileValue;
    type IntoIter = std::slice::Iter<'a, SaveFileValue>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.inner).into_iter()
    }
}

impl<T: GameObjectCollection> Debug for GameObject<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GameObject({},{:?})", self.name, self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut obj = GameObjectMap::from_name("test".to_owned());
        let val = GameString::wrap("value".to_owned());
        obj.insert("key".to_owned(), SaveFileValue::String(val.clone()));
        assert_eq!(obj.get("key").unwrap().as_string().unwrap(), val.clone());
        let val2 = GameString::wrap("value2".to_owned());
        obj.insert("key".to_owned(), SaveFileValue::String(val2.clone()));
        let arr = obj
            .get("key")
            .unwrap()
            .as_object()
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(arr.len(), 2);
    }
}
