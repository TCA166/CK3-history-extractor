use std::{
    any::type_name,
    error,
    fmt::{self, Debug, Display},
    num::{ParseFloatError, ParseIntError},
    rc::Rc,
    str::{FromStr, ParseBoolError},
};

use jomini::common::Date;

use super::super::types::{HashMap, RefOrRaw, Wrapper};

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
    InvalidValue(&'static str),
}

impl From<ParseIntError> for ConversionError {
    fn from(_: ParseIntError) -> Self {
        ConversionError::InvalidValue("integer")
    }
}

impl From<ParseFloatError> for ConversionError {
    fn from(_: ParseFloatError) -> Self {
        ConversionError::InvalidValue("float")
    }
}

impl From<ParseBoolError> for ConversionError {
    fn from(_: ParseBoolError) -> Self {
        ConversionError::InvalidValue("bool")
    }
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidType(t1, t2) => write!(f, "failed converting {:?} to {}", t1, t2),
            Self::InvalidValue(desc) => write!(f, "the value is invalid for conversion {}", desc),
        }
    }
}

impl error::Error for ConversionError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

const BOOL_TRUE: &str = "yes";
const BOOL_FALSE: &str = "no";

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
    Date(Date),
}

impl FromStr for SaveFileValue {
    type Err = ConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dot_count = s.chars().filter(|&c| c == '.').count();
        if dot_count == 2 {
            let mut parts = s.split('.');
            if let Ok(year) = parts.next().unwrap().parse() {
                if let Ok(month) = parts.next().unwrap().parse() {
                    if let Ok(day) = parts.next().unwrap().parse() {
                        return Ok(Date::from_ymd_opt(year, month, day)
                            .ok_or(ConversionError::InvalidValue("invalid date components"))?
                            .into());
                    }
                }
            }
        } else if dot_count == 1 {
            if let Ok(f) = s.parse() {
                return Ok(SaveFileValue::Real(f));
            }
        } else if dot_count == 0 {
            if s == BOOL_TRUE {
                return Ok(SaveFileValue::Boolean(true));
            } else if s == BOOL_FALSE {
                return Ok(SaveFileValue::Boolean(false));
            } else if let Ok(int) = s.parse() {
                return Ok(SaveFileValue::Integer(int));
            }
        }
        Ok(SaveFileValue::String(Rc::new(s.to_owned())))
    }
}

impl From<String> for SaveFileValue {
    fn from(value: String) -> Self {
        SaveFileValue::String(Rc::new(value))
    }
}

impl From<i64> for SaveFileValue {
    fn from(value: i64) -> Self {
        SaveFileValue::Integer(value)
    }
}

impl From<i32> for SaveFileValue {
    fn from(value: i32) -> Self {
        SaveFileValue::Integer(value as i64)
    }
}

impl From<bool> for SaveFileValue {
    fn from(value: bool) -> Self {
        SaveFileValue::Boolean(value)
    }
}

impl From<u32> for SaveFileValue {
    fn from(value: u32) -> Self {
        SaveFileValue::Integer(value as i64)
    }
}

impl From<u64> for SaveFileValue {
    fn from(value: u64) -> Self {
        SaveFileValue::Integer(value as i64)
    }
}

impl From<SaveFileObject> for SaveFileValue {
    fn from(value: SaveFileObject) -> Self {
        SaveFileValue::Object(value)
    }
}

impl From<[u8; 4]> for SaveFileValue {
    fn from(value: [u8; 4]) -> Self {
        SaveFileValue::Real(f32::from_le_bytes(value) as f64)
    }
}

impl From<[u8; 8]> for SaveFileValue {
    fn from(value: [u8; 8]) -> Self {
        SaveFileValue::Real(f64::from_le_bytes(value))
    }
}

impl From<Date> for SaveFileValue {
    fn from(value: Date) -> Self {
        SaveFileValue::Date(value)
    }
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
            SaveFileValue::Real(r) => Ok(*r as i64),
            _ => Err(ConversionError::InvalidType(
                self.clone(),
                type_name::<i64>(),
            )),
        }
    }

    pub fn as_real(&self) -> Result<f64, ConversionError> {
        match self {
            SaveFileValue::Real(r) => Ok(*r),
            SaveFileValue::Integer(i) => Ok(*i as f64),
            _ => Err(ConversionError::InvalidType(
                self.clone(),
                type_name::<f64>(),
            )),
        }
    }

    pub fn as_date(&self) -> Result<Date, ConversionError> {
        match self {
            SaveFileValue::Date(date) => Ok(*date),
            _ => Err(ConversionError::InvalidType(
                self.clone(),
                type_name::<(i16, u8, u8)>(),
            )),
        }
    }

    pub fn as_boolean(&self) -> Result<bool, ConversionError> {
        match self {
            SaveFileValue::Boolean(b) => Ok(*b),
            _ => Err(ConversionError::InvalidType(
                self.clone(),
                type_name::<bool>(),
            )),
        }
    }
}

/// A game object that stores values as a map.
pub type GameObjectMap = HashMap<String, SaveFileValue>;
/// A game object that stores values as an array.
pub type GameObjectArray = Vec<SaveFileValue>;

/// An object that comes from a save file.
#[derive(PartialEq, Clone, Debug)]
pub enum SaveFileObject {
    /// An object that stores values as a map.
    Map(GameObjectMap),
    /// An object that stores values as an array.
    Array(GameObjectArray),
}

impl SaveFileObject {
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

    /// Check if the object is empty
    pub fn is_empty(&self) -> bool {
        match self {
            SaveFileObject::Map(m) => m.is_empty(),
            SaveFileObject::Array(a) => a.is_empty(),
        }
    }
}

#[derive(Debug)]
pub enum SaveObjectError {
    ConversionError(ConversionError),
    KeyError(KeyError),
}

pub trait GameObjectMapping {
    fn get_err(&self, key: &str) -> Result<&SaveFileValue, KeyError>;
    fn get_string(&self, key: &str) -> Result<GameString, SaveObjectError>;
    fn get_object(&self, key: &str) -> Result<&SaveFileObject, SaveObjectError>;
    fn get_integer(&self, key: &str) -> Result<i64, SaveObjectError>;
    fn get_real(&self, key: &str) -> Result<f64, SaveObjectError>;
    fn get_game_id(&self, key: &str) -> Result<GameId, SaveObjectError>;
    fn get_date(&self, key: &str) -> Result<Date, SaveObjectError>;
}

impl GameObjectMapping for GameObjectMap {
    fn get_err(&self, key: &str) -> Result<&SaveFileValue, KeyError> {
        self.get(key) // lazy error initialization, else we copy key and obj every time
            .ok_or_else(|| KeyError::MissingKey(key.to_owned(), self.clone()))
    }

    fn get_string(&self, key: &str) -> Result<GameString, SaveObjectError> {
        Ok(self.get_err(key)?.as_string()?)
    }

    /// Get the value of a key as a boolean.
    fn get_object(&self, key: &str) -> Result<&SaveFileObject, SaveObjectError> {
        Ok(self.get_err(key)?.as_object()?)
    }

    /// Get the value of a key as an integer.
    fn get_integer(&self, key: &str) -> Result<i64, SaveObjectError> {
        Ok(self.get_err(key)?.as_integer()?)
    }

    /// Get the value of a key as a real number.
    fn get_real(&self, key: &str) -> Result<f64, SaveObjectError> {
        Ok(self.get_err(key)?.as_real()?)
    }

    /// Get the value of a key as a GameId.
    fn get_game_id(&self, key: &str) -> Result<GameId, SaveObjectError> {
        Ok(self.get_err(key)?.as_id()?)
    }

    fn get_date(&self, key: &str) -> Result<Date, SaveObjectError> {
        Ok(self.get_err(key)?.as_date()?)
    }
}

pub trait GameObjectCollection {
    fn get_index(&self, index: usize) -> Result<&SaveFileValue, KeyError>;
}

impl GameObjectCollection for GameObjectArray {
    fn get_index(&self, index: usize) -> Result<&SaveFileValue, KeyError> {
        self.get(index)
            .ok_or_else(|| KeyError::IndexError(index, self.clone()))
    }
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
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingKey(key, obj) => write!(f, "key {} missing from object {:?}", key, obj),
            Self::IndexError(index, obj) => write!(f, "index {} out of range for {:?}", index, obj),
        }
    }
}

impl error::Error for KeyError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
