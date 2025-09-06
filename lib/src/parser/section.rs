use derive_more::{Display, From};
use jomini::{
    binary::{ReaderError as BinaryReaderError, Token as BinaryToken, TokenResolver},
    text::{ReaderError as TextReaderError, Token as TextToken},
    Scalar, ScalarError,
};

use super::{
    super::types::HashMap,
    game_object::ConversionError,
    tokens::TOKEN_TRANSLATOR,
    types::{Tape, TapeError},
    SaveFileObject, SaveFileValue,
};

use std::{
    error,
    fmt::{self, Debug},
    num::ParseIntError,
    string::FromUtf8Error,
};

/// An error that occured while processing a specific section
#[derive(Debug, From, Display)]
pub enum SectionError {
    /// An error occured while converting a value
    ConversionError(ConversionError),
    /// An error occured while parsing a scalar
    ScalarError(ScalarError),
    /// An unknown token was encountered
    UnknownToken(u16),
    /// An error occured while reading from the tape
    TapeError(TapeError),
    /// An error occured while decoding bytes
    DecodingError(FromUtf8Error),
}

impl From<TextReaderError> for SectionError {
    fn from(value: TextReaderError) -> Self {
        Self::TapeError(value.into())
    }
}

impl From<BinaryReaderError> for SectionError {
    fn from(value: BinaryReaderError) -> Self {
        Self::TapeError(value.into())
    }
}

impl From<ParseIntError> for SectionError {
    fn from(value: ParseIntError) -> Self {
        Self::ConversionError(value.into())
    }
}

impl error::Error for SectionError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::ConversionError(err) => Some(err),
            Self::ScalarError(err) => Some(err),
            Self::TapeError(err) => Some(err),
            Self::DecodingError(err) => Some(err),
            _ => None,
        }
    }
}

/// The headers preceding color values. To be ignored
const COLOR_HEADERS: [&[u8]; 2] = [b"rgb", b"hsv"];

/// A stack entry for the section parser.
/// It serves two very important functions. First: it stores the name it should
/// be saved under, or 'None' if it should be saved in parent as if the parent
/// was an array. Second: it stores the values that are being parsed,
/// as if the object was simultaneously an array and a map. This is then
/// lazily evaluated into a homogeneous object. The object internals are lazily
/// evaluated so performance cost for homogenous objects should be minimal
#[derive(Debug, Clone)]
struct StackEntry {
    name: Option<String>,
    array: Option<Vec<SaveFileValue>>,
    map: Option<HashMap<String, SaveFileValue>>,
}

impl StackEntry {
    /// Create a new stack entry with an optional name.
    fn new(name: Option<String>) -> Self {
        StackEntry {
            name,
            array: None,
            map: None,
        }
    }

    /// Push a value into the stack entry.
    fn push(&mut self, value: SaveFileValue) {
        if self.array.is_none() {
            self.array = Some(Vec::new());
        }
        self.array.as_mut().unwrap().push(value);
    }

    /// Insert a key-value pair into the stack entry.
    fn insert(&mut self, key: String, value: SaveFileValue) {
        if self.map.is_none() {
            self.map = Some(HashMap::new());
        }
        let map = self.map.as_mut().unwrap();
        if let Some(val) = map.get_mut(&key) {
            if let SaveFileValue::Object(ob) = val {
                if let SaveFileObject::Array(arr) = ob {
                    arr.push(value);
                    return;
                }
            }
            let arr = vec![val.clone(), value];
            map.insert(key, SaveFileValue::Object(SaveFileObject::Array(arr)));
        } else {
            map.insert(key, value);
        }
    }
}

impl Into<SaveFileObject> for StackEntry {
    fn into(self) -> SaveFileObject {
        if self.map.is_none() {
            return SaveFileObject::Array(self.array.unwrap_or(Vec::new()));
        } else if self.array.is_none() {
            return SaveFileObject::Map(self.map.unwrap());
        } else {
            let mut map = self.map.unwrap();
            let mut array = self.array.unwrap();
            // now we have to somehow combine universally a hashmap and an array
            if map.keys().all(|k| k.chars().all(|k| k.is_digit(10))) {
                // the map keys are all numerical, means probably we want to treat them as indices into the array
                let mut keys = map
                    .keys()
                    .map(|k| (k.parse::<usize>().unwrap(), k.clone()))
                    .collect::<Vec<_>>();
                keys.sort();
                for (index, key) in keys {
                    let value = map.remove(&key).unwrap();
                    if index > array.len() {
                        array.push(value);
                    } else {
                        array.insert(index, value);
                    }
                }
                return SaveFileObject::Array(array);
            } else {
                unimplemented!(
                    "combining a hashmap and an array is not yet implemented, {:?}, {:?}",
                    map,
                    array
                );
            }
        }
    }
}

/// Process a scalar into a string.
/// The [ToString] implementation of [Scalar] will be used if the scalar is ASCII.
/// This implementation is weird overall because it will not handle non-ASCII characters correctly.
fn scalar_to_string(scalar: Scalar) -> Result<String, SectionError> {
    if scalar.is_ascii() {
        Ok(scalar.to_string())
    } else {
        // TODO optimalization in which you can avoid parsing the string if the string is non ascii
        Ok(String::from_utf8(scalar.as_bytes().to_vec())?)
    }
}

/// A section of the save file.
/// It directly maps to a [SaveFileObject] and is the largest unit of data in the save file.
pub struct Section<'tape, 'data> {
    tape: &'tape mut Tape<'data>,
    name: String,
}

impl<'tape, 'data> Section<'tape, 'data> {
    /// Create a new section from a tape.
    /// The section will be named `name` and will start at `offset` and end at `end`.
    /// The first token of the section (pointed at by `offset`) is expected to an object or array token.
    /// The end token is not included in the section.
    pub fn new(tape: &'tape mut Tape<'data>, name: String) -> Self {
        Section { tape, name }
    }

    /// Get the name of the section.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Skip the section. This must be called if the section is not going to be parsed.
    pub fn skip(self) -> Result<(), SectionError> {
        Ok(self.tape.skip_container()?)
    }

    /// Parse the section into a [SaveFileObject]. This will consume the section.
    /// Since the tape holds state, it must be mutable for the section to be parsable.
    pub fn parse(self) -> Result<SaveFileObject, SectionError> {
        let mut stack: Vec<StackEntry> = vec![StackEntry::new(Some(self.name.clone()))];
        let mut key = None;
        let mut past_eq = false;
        /// Blanket implementation that handles a new token, assuming that the token cannot be a key
        fn add_value_quoted<T: Into<SaveFileValue>>(
            stack: &mut Vec<StackEntry>,
            key: &mut Option<String>,
            past_eq: &mut bool,
            token: T,
        ) {
            if *past_eq {
                stack
                    .last_mut()
                    .unwrap()
                    .insert(key.take().unwrap(), token.into());
                *past_eq = false;
            } else {
                stack.last_mut().unwrap().push(token.into());
            }
        }
        /// Blanket implementation that handles a new token while making no assumptions
        fn add_value_unquoted<T: Into<SaveFileValue> + ToString>(
            stack: &mut Vec<StackEntry>,
            key: &mut Option<String>,
            past_eq: &mut bool,
            token: T,
        ) {
            if *past_eq {
                stack
                    .last_mut()
                    .unwrap()
                    .insert(key.take().unwrap(), token.into());
                *past_eq = false;
            } else {
                if let Some(key) = key.replace(token.to_string()) {
                    stack.last_mut().unwrap().push(key.into());
                }
            }
        }
        match self.tape {
            Tape::Text(text) => {
                while let Some(result) = text.next().transpose() {
                    match result {
                        Err(e) => return Err(e.into()),
                        Ok(tok) => match tok {
                            TextToken::Open => {
                                stack.push(StackEntry::new(key.take()));
                                if past_eq {
                                    past_eq = false;
                                }
                            }
                            TextToken::Close => {
                                let mut last = stack.pop().unwrap();
                                if let Some(key) = key.take() {
                                    last.push(key.into());
                                }
                                let name = last.name.take();
                                let value: SaveFileObject = last.into();
                                if let Some(entry) = stack.last_mut() {
                                    if name.is_some() {
                                        entry.insert(name.unwrap(), value.into());
                                    } else {
                                        entry.push(value.into());
                                    }
                                } else {
                                    return Ok(value);
                                }
                            }
                            TextToken::Operator(_op) => {
                                // here we have a problem, when parsing game code, there can be some instances of non = operators
                                // MAYBE solve this here somehow
                                /*
                                if op == Operator::Equal {
                                    past_eq = true;
                                } else {
                                    past_eq = false;
                                } */
                                past_eq = true;
                            }
                            TextToken::Quoted(token) => {
                                add_value_quoted(
                                    &mut stack,
                                    &mut key,
                                    &mut past_eq,
                                    scalar_to_string(token)?,
                                );
                            }
                            TextToken::Unquoted(token) => {
                                // zero cost operation
                                if COLOR_HEADERS.contains(&token.as_bytes()) {
                                    continue; // we want to skip an unquoted token in situations like this: `color=rgb { 255 255 255 }`
                                }
                                add_value_unquoted(
                                    &mut stack,
                                    &mut key,
                                    &mut past_eq,
                                    scalar_to_string(token)?,
                                );
                            }
                        },
                    }
                }
            }
            Tape::Binary(binary) => {
                while let Some(result) = binary.next().transpose() {
                    match result {
                        Err(e) => return Err(e.into()),
                        Ok(tok) => match tok {
                            BinaryToken::Open => {
                                stack.push(StackEntry::new(key.take()));
                                if past_eq {
                                    past_eq = false;
                                }
                            }
                            BinaryToken::Close => {
                                let mut last = stack.pop().unwrap();
                                if let Some(key) = key.take() {
                                    last.push(key.into());
                                }
                                let name = last.name.take();
                                let value: SaveFileObject = last.into();
                                if let Some(entry) = stack.last_mut() {
                                    if name.is_some() {
                                        entry.insert(name.unwrap(), value.into());
                                    } else {
                                        entry.push(value.into());
                                    }
                                } else {
                                    return Ok(value.into());
                                }
                            }
                            BinaryToken::Equal => {
                                past_eq = true;
                            }
                            BinaryToken::Quoted(token) => {
                                add_value_unquoted(
                                    &mut stack,
                                    &mut key,
                                    &mut past_eq,
                                    scalar_to_string(token)?,
                                );
                            }
                            BinaryToken::Unquoted(token) => {
                                add_value_unquoted(
                                    &mut stack,
                                    &mut key,
                                    &mut past_eq,
                                    scalar_to_string(token)?,
                                );
                            }
                            BinaryToken::Bool(token) => {
                                add_value_unquoted(&mut stack, &mut key, &mut past_eq, token);
                            }
                            BinaryToken::I32(token) => {
                                add_value_unquoted(&mut stack, &mut key, &mut past_eq, token);
                            }
                            BinaryToken::I64(token) => {
                                add_value_unquoted(&mut stack, &mut key, &mut past_eq, token);
                            }
                            BinaryToken::F32(token) => {
                                add_value_quoted(&mut stack, &mut key, &mut past_eq, token);
                            }
                            BinaryToken::F64(token) => {
                                add_value_quoted(&mut stack, &mut key, &mut past_eq, token);
                            }
                            BinaryToken::Id(token) => {
                                let resolved = TOKEN_TRANSLATOR
                                    .resolve(token)
                                    .ok_or(SectionError::UnknownToken(token))?
                                    .to_string();
                                add_value_unquoted(&mut stack, &mut key, &mut past_eq, resolved);
                            }
                            BinaryToken::Rgb(token) => {
                                let value = SaveFileObject::Array(vec![
                                    token.r.into(),
                                    token.g.into(),
                                    token.b.into(),
                                ]);
                                add_value_quoted(&mut stack, &mut key, &mut past_eq, value);
                            }
                            BinaryToken::U32(token) => {
                                add_value_unquoted(&mut stack, &mut key, &mut past_eq, token);
                            }
                            BinaryToken::U64(token) => {
                                add_value_unquoted(&mut stack, &mut key, &mut past_eq, token);
                            }
                        },
                    }
                }
            }
        }
        return Ok(stack.pop().unwrap().into());
    }
}

impl Debug for Section<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Section").field("name", &self.name).finish()
    }
}

#[cfg(test)]
mod tests {

    use jomini::text::TokenReader;

    use super::*;

    use super::super::types::Tape;

    #[test]
    fn test_empty() {
        let mut tape = Tape::Text(TokenReader::from_slice(b""));
        let section = Section::new(&mut tape, "empty".to_string());
        assert_eq!(section.get_name(), "empty");
        let obj = section.parse().unwrap();
        assert!(matches!(obj, SaveFileObject::Array(_)));
    }

    #[test]
    fn test_mixed_obj() {
        let mut tape = Tape::Text(TokenReader::from_slice(b"a b 1=c 2={d=5}}"));
        let section = Section::new(&mut tape, "test".to_string());
        let obj = section.parse();
        assert!(obj.is_ok());
        let res = obj.unwrap();
        if let SaveFileObject::Array(arr) = res {
            assert_eq!(arr.len(), 4);
            let obj = arr.get(2).unwrap();
            if let SaveFileValue::Object(obj) = obj {
                assert_eq!(
                    obj.as_map()
                        .unwrap()
                        .get("d")
                        .unwrap()
                        .as_integer()
                        .unwrap(),
                    5
                );
            } else {
                panic!("expected object");
            }
        } else {
            panic!("expected array");
        }
    }

    #[test]
    fn test_mixed_duplicate_keys() {
        let mut tape = Tape::Text(TokenReader::from_slice(b"a b 1=c 2={d=5} 1={e=6}"));
        let section = Section::new(&mut tape, "test".to_string());
        let obj = section.parse().unwrap();
        obj.as_array()
            .unwrap()
            .get(1)
            .unwrap()
            .as_object()
            .unwrap()
            .as_array()
            .unwrap();
    }

    #[test]
    fn test_rgb() {
        let mut tape = Tape::Text(TokenReader::from_slice(b"color1=rgb { 220 220 220 }"));
        let section = Section::new(&mut tape, "test".to_string());
        let obj = section.parse().unwrap();
        let rgb = obj
            .as_map()
            .unwrap()
            .get("color1")
            .unwrap()
            .as_object()
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(rgb.len(), 3);
    }

    #[test]
    fn test_skip() {
        let mut tape = Tape::Text(TokenReader::from_slice(b"color1=rgb { 220 220 220 }} "));
        let section = Section::new(&mut tape, "test".to_string());
        section.skip().unwrap();

        assert_eq!(tape.position(), 27)
    }

    #[test]
    fn test_utf8() {
        let mut tape = Tape::Text(TokenReader::from_slice(
            "test=\"Malik al-Muazzam Styrkár\"}".as_bytes(),
        ));
        let section = Section::new(&mut tape, "test".to_string());
        let obj = section.parse().unwrap();
        let utf8 = obj
            .as_map()
            .unwrap()
            .get("test")
            .unwrap()
            .as_string()
            .unwrap();
        assert_eq!(utf8.as_ref(), "Malik al-Muazzam Styrkár");
    }
}
