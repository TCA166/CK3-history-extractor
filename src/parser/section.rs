use jomini::{
    binary::{ReaderError as BinaryReaderError, Token as BinaryToken},
    text::{Operator, ReaderError as TextReaderError, Token as TextToken},
    Scalar, ScalarError,
};

use super::{
    game_object::ConversionError,
    types::{Tape, TapeError},
    SaveFileObject, SaveFileValue,
};

use std::{
    collections::HashMap,
    error,
    fmt::{self, Debug, Display},
    num::ParseIntError,
};

/// An error that occured while processing a specific section
#[derive(Debug)]
pub enum SectionError {
    /// An error occured while converting a value
    ConversionError(ConversionError),
    /// An error occured while parsing a scalar
    ScalarError(ScalarError),
    /// An unknown token was encountered
    UnknownToken(u16),
    TapeError(TapeError),
}

impl Display for SectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConversionError(err) => Display::fmt(err, f),
            Self::ScalarError(err) => Display::fmt(err, f),
            Self::UnknownToken(tok) => write!(f, "unknown token {}", tok),
            Self::TapeError(err) => Display::fmt(err, f),
        }
    }
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

impl From<TapeError> for SectionError {
    fn from(value: TapeError) -> Self {
        Self::TapeError(value)
    }
}

impl From<ConversionError> for SectionError {
    fn from(value: ConversionError) -> Self {
        Self::ConversionError(value)
    }
}

impl From<ParseIntError> for SectionError {
    fn from(value: ParseIntError) -> Self {
        Self::ConversionError(value.into())
    }
}

impl From<ScalarError> for SectionError {
    fn from(value: ScalarError) -> Self {
        Self::ScalarError(value)
    }
}

impl error::Error for SectionError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::ConversionError(err) => Some(err),
            Self::ScalarError(err) => Some(err),
            Self::TapeError(err) => Some(err),
            _ => None,
        }
    }
}

fn token_resolver(token: &u16) -> Option<&'static str> {
    match token {
        // TODO
        _ => None,
    }
}

#[derive(Debug, Clone)]
struct StackEntry {
    name: Option<String>,
    array: Option<Vec<SaveFileValue>>,
    map: Option<HashMap<String, SaveFileValue>>,
}

impl StackEntry {
    fn new(name: Option<String>) -> Self {
        StackEntry {
            name,
            array: None,
            map: None,
        }
    }

    fn push(&mut self, value: SaveFileValue) {
        if self.array.is_none() {
            self.array = Some(Vec::new());
        }
        self.array.as_mut().unwrap().push(value);
    }

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

/// Convert a stack entry into a [SaveFileObject].
/// This function will consume the stack entry.
/// If the stack entry contains both an array and a map, the map will be treated as an array index.
/// [StackEntry] doesn't implement [Into]<[SaveFileObject]> because it would be a partial conversion.
fn stack_entry_into_object<'a>(entry: &mut StackEntry) -> SaveFileObject {
    if entry.map.is_none() {
        return SaveFileObject::Array(entry.array.take().unwrap_or(Vec::new()));
    } else if entry.array.is_none() {
        return SaveFileObject::Map(entry.map.take().unwrap());
    } else {
        let mut map = entry.map.take().unwrap();
        let mut array = entry.array.take().unwrap();
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
            unimplemented!("combining a hashmap and an array is not yet implemented");
        }
    }
}

// here's how we handle utf8 strings:
impl From<Scalar<'_>> for SaveFileValue {
    fn from(value: Scalar) -> Self {
        let str = if value.is_ascii() {
            value.to_string()
        } else {
            std::str::from_utf8(value.as_bytes()).unwrap().to_string()
        };
        SaveFileValue::String(str.into())
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

    pub fn skip(&mut self) -> Result<(), SectionError> {
        Ok(self.tape.skip_container()?)
    }

    /* Looking at this parser, you can quite easily see an opportunity for an
    abstraction based on an enum. This would allow us to have a single parser
    that can handle both text and binary tokens. The problem with that approach
    is that it would abstract too much. */

    /// Parse the section into a [SaveFileObject].
    pub fn parse(&mut self) -> Result<SaveFileObject, SectionError> {
        let mut stack: Vec<StackEntry> = vec![StackEntry::new(Some(self.name.clone()))];
        let mut key = None;
        let mut past_eq = false;
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
                                if key.is_some() {
                                    last.push(SaveFileValue::String(key.take().unwrap().into()));
                                }
                                let name = last.name.take();
                                let value = stack_entry_into_object(&mut last);
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
                            TextToken::Operator(op) => {
                                if op == Operator::Equal {
                                    past_eq = true;
                                } else {
                                    past_eq = false;
                                }
                            }
                            TextToken::Quoted(token) => {
                                if past_eq {
                                    stack
                                        .last_mut()
                                        .unwrap()
                                        .insert(key.take().unwrap(), token.into());
                                    past_eq = false;
                                } else {
                                    stack.last_mut().unwrap().push(token.into());
                                }
                            }
                            TextToken::Unquoted(token) => {
                                if ["rgb", "hsv"].contains(&token.to_string().as_str()) {
                                    continue;
                                }
                                if past_eq {
                                    stack
                                        .last_mut()
                                        .unwrap()
                                        .insert(key.take().unwrap(), token.into());
                                    past_eq = false;
                                } else {
                                    if let Some(key) = key {
                                        stack
                                            .last_mut()
                                            .unwrap()
                                            .push(SaveFileValue::String(key.into()));
                                    }
                                    key = Some(token.to_string());
                                }
                            }
                        },
                    }
                }
            }
            Tape::Binary(binary) => {
                fn add_value<T: Into<SaveFileValue>>(
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
                                let name = last.name.take();
                                let value = stack_entry_into_object(&mut last);
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
                                add_value(&mut stack, &mut key, &mut past_eq, token);
                            }
                            BinaryToken::Unquoted(token) => {
                                add_value(&mut stack, &mut key, &mut past_eq, token);
                            }
                            BinaryToken::Bool(token) => {
                                add_value(&mut stack, &mut key, &mut past_eq, token);
                            }
                            BinaryToken::I32(token) => {
                                add_value(&mut stack, &mut key, &mut past_eq, token);
                            }
                            BinaryToken::I64(token) => {
                                add_value(&mut stack, &mut key, &mut past_eq, token);
                            }
                            BinaryToken::F32(token) => {
                                add_value(&mut stack, &mut key, &mut past_eq, token);
                            }
                            BinaryToken::F64(token) => {
                                add_value(&mut stack, &mut key, &mut past_eq, token);
                            }
                            BinaryToken::Id(token) => {
                                let str = token_resolver(&token)
                                    .ok_or(SectionError::UnknownToken(token))?;
                                key = Some(str.to_string());
                            }
                            BinaryToken::Rgb(token) => {
                                let value = SaveFileObject::Array(vec![
                                    token.r.into(),
                                    token.g.into(),
                                    token.b.into(),
                                ]);
                                add_value(&mut stack, &mut key, &mut past_eq, value);
                            }
                            BinaryToken::U32(token) => {
                                add_value(&mut stack, &mut key, &mut past_eq, token);
                            }
                            BinaryToken::U64(token) => {
                                add_value(&mut stack, &mut key, &mut past_eq, token);
                            }
                        },
                    }
                }
            }
        }
        return Ok(stack_entry_into_object(&mut stack.pop().unwrap()));
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
        let mut section = Section::new(&mut tape, "empty".to_string());
        assert_eq!(section.get_name(), "empty");
        let obj = section.parse().unwrap();
        assert!(matches!(obj, SaveFileObject::Array(_)));
    }

    #[test]
    fn test_mixed_obj() {
        let mut tape = Tape::Text(TokenReader::from_slice(b"a b 1=c 2={d=5}}"));
        let mut section = Section::new(&mut tape, "test".to_string());
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
        let mut section = Section::new(&mut tape, "test".to_string());
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
        let mut section = Section::new(&mut tape, "test".to_string());
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
        let mut section = Section::new(&mut tape, "test".to_string());
        section.skip().unwrap();

        assert_eq!(tape.position(), 27)
    }

    #[test]
    fn test_utf8() {
        let mut tape = Tape::Text(TokenReader::from_slice(
            "test=\"Malik al-Muazzam Styrkár\"}".as_bytes(),
        ));
        let mut section = Section::new(&mut tape, "test".to_string());
        let obj = section.parse().unwrap();
        let utf8 = obj
            .as_map()
            .unwrap()
            .get("test")
            .unwrap()
            .as_string()
            .unwrap();
        assert_eq!(utf8.as_str(), "Malik al-Muazzam Styrkár");
    }
}
