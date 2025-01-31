use jomini::{
    binary::{ReaderError as BinaryReaderError, Token as BinaryToken},
    text::{Operator, ReaderError as TextReaderError, Token as TextToken},
    ScalarError,
};

use super::{
    super::types::Wrapper,
    game_object::ConversionError,
    types::{Tape, Token},
    GameObjectArray, GameObjectMap, GameString, SaveFileObject, SaveFileValue,
};

use std::{
    collections::HashMap,
    error,
    fmt::{self, Debug, Display},
    mem,
    num::ParseIntError,
};

/// An error that occured while processing a specific section
#[derive(Debug)]
pub enum SectionError<'a> {
    /// A token was in some way unexpected
    UnexpectedToken(usize, Token<'a>, &'static str),
    /// An error occured while converting a value
    ConversionError(ConversionError),
    /// An error occured while parsing a scalar
    ScalarError(ScalarError),
    /// An unknown token was encountered
    UnknownToken(u16),
    TextReaderError(TextReaderError),
    BinaryReaderError(BinaryReaderError),
}

impl<'a> Display for SectionError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken(pos, tok, desc) => {
                write!(f, "token {:?} at {} is unexpected: {}", tok, pos, desc)
            }
            Self::ConversionError(err) => Display::fmt(err, f),
            Self::ScalarError(err) => Display::fmt(err, f),
            Self::UnknownToken(tok) => write!(f, "unknown token {}", tok),
            Self::TextReaderError(err) => Display::fmt(err, f),
            Self::BinaryReaderError(err) => Display::fmt(err, f),
        }
    }
}

impl From<TextReaderError> for SectionError<'_> {
    fn from(value: TextReaderError) -> Self {
        Self::TextReaderError(value)
    }
}

impl From<BinaryReaderError> for SectionError<'_> {
    fn from(value: BinaryReaderError) -> Self {
        Self::BinaryReaderError(value)
    }
}

impl From<ConversionError> for SectionError<'_> {
    fn from(value: ConversionError) -> Self {
        Self::ConversionError(value)
    }
}

impl From<ParseIntError> for SectionError<'_> {
    fn from(value: ParseIntError) -> Self {
        Self::ConversionError(value.into())
    }
}

impl From<ScalarError> for SectionError<'_> {
    fn from(value: ScalarError) -> Self {
        Self::ScalarError(value)
    }
}

impl<'a> error::Error for SectionError<'a> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::ConversionError(err) => Some(err),
            Self::ScalarError(err) => Some(err),
            Self::TextReaderError(err) => Some(err),
            Self::BinaryReaderError(err) => Some(err),
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

struct StackEntry {
    name: Option<String>,
    pub array: Vec<SaveFileValue>,
    pub map: HashMap<String, SaveFileValue>,
    pub past_eq: bool,
}

impl Into<SaveFileObject> for StackEntry {
    fn into(mut self) -> SaveFileObject {
        if self.map.is_empty() {
            return SaveFileObject::Array(GameObjectArray::new(self.name, self.array));
        } else if self.array.is_empty() {
            return SaveFileObject::Map(GameObjectMap::new(self.name, self.map));
        } else {
            // now we have to somehow combine universally a hashmap and an array
            if self.map.keys().all(|k| k.chars().all(|k| k.is_digit(10))) {
                // the map keys are all numerical, means probably we want to treat them as indices into the array
                for (key, value) in self.map {
                    let index = key.parse::<usize>().unwrap();
                    if index > self.array.len() {
                        self.array.push(value);
                    } else {
                        self.array.insert(index, value);
                    }
                }
                return SaveFileObject::Array(GameObjectArray::new(self.name, self.array));
            } else {
                panic!("what");
            }
        }
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

    /* Looking at this parser, you can quite easily see an opportunity for an
    abstraction based on an enum. This would allow us to have a single parser
    that can handle both text and binary tokens. The problem with that approach
    is that it would abstract too much. */

    /// Parse the section into a [SaveFileObject].
    pub fn parse(&mut self) -> Result<SaveFileObject, SectionError> {
        let mut stack: Vec<StackEntry> = vec![StackEntry {
            name: Some(self.name.clone()),
            array: Vec::default(),
            map: HashMap::default(),
            past_eq: true,
        }];
        let mut key = None;
        let mut past_eq = false;
        match self.tape {
            Tape::Text(text) => {
                while let Some(result) = text.next().transpose() {
                    match result {
                        Err(e) => return Err(e.into()),
                        Ok(tok) => match tok {
                            TextToken::Open => stack.push(StackEntry {
                                name: mem::take(&mut key),
                                array: Vec::default(),
                                map: HashMap::default(),
                                past_eq,
                            }),
                            TextToken::Close => {}
                            TextToken::Operator(op) => {
                                if op == Operator::Equal {
                                    past_eq = true;
                                } else {
                                    past_eq = false;
                                }
                            }
                            TextToken::Quoted(token) => {
                                if past_eq {
                                    if let Some(entry) = stack.last_mut() {
                                        entry.array.push(token.to_string().into());
                                    } else {
                                        return Err(SectionError::UnexpectedToken(
                                            text.position(),
                                            tok.clone().into(),
                                            "stack is empty",
                                        ));
                                    }
                                    past_eq = false;
                                } else {
                                }
                            }
                            TextToken::Unquoted(token) => {}
                        },
                    }
                }
            }
            Tape::Binary(binary) => {
                while let Some(result) = binary.next().transpose() {
                    match result {
                        Err(e) => return Err(e.into()),
                        Ok(tok) => match tok {
                            BinaryToken::Open => {}
                            BinaryToken::Close => {}
                            BinaryToken::Equal => {}
                            BinaryToken::Quoted(token) => {}
                            BinaryToken::Unquoted(token) => {}
                            BinaryToken::Bool(token) => {}
                            BinaryToken::I32(token) => {}
                            BinaryToken::I64(token) => {}
                            BinaryToken::F32(token) => {}
                            BinaryToken::F64(token) => {}
                            BinaryToken::I32(token) => {}
                            BinaryToken::I64(token) => {}
                            BinaryToken::Id(token) => {}
                            BinaryToken::Rgb(token) => {}
                            BinaryToken::U32(token) => {}
                            BinaryToken::U64(token) => {}
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
        let mut section = Section::new(&mut tape, "empty".to_string());
        let obj = section.parse().unwrap();
        assert_eq!(obj.get_name().unwrap(), "empty");
        assert!(matches!(obj, SaveFileObject::Map(_)));
    }

    #[test]
    fn test_mixed_obj() {
        let mut tape = Tape::Text(TokenReader::from_slice(b"test={a b 1=c 2={d=5}}"));
        let mut section = Section::new(&mut tape, "test".to_string());
        let obj = section.parse();
        assert!(obj.is_ok());
    }

    #[test]
    fn test_mixed() {
        let mut tape = Tape::Text(TokenReader::from_slice(b"test={a b 1=c 2=d}"));
        let mut section = Section::new(&mut tape, "test".to_string());
        let obj = section.parse().unwrap();
        assert_eq!(obj.get_name().unwrap(), "test");
        if let SaveFileObject::Array(arr) = obj {
            assert_eq!(arr.len(), 4);
        } else {
            panic!("expected array");
        }
    }
}
