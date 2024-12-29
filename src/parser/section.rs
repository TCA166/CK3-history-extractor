use jomini::{text::Operator, BinaryToken, ScalarError, TextToken};

use super::{
    super::types::Wrapper,
    game_object::ConversionError,
    types::{Token, Tokens},
    GameObjectArray, GameObjectMap, GameString, SaveFileObject, SaveFileValue,
};

use std::{
    error,
    fmt::{self, Debug, Display},
    num::ParseIntError,
};

/// An error that occured while processing a specific section
#[derive(Debug)]
pub enum SectionError<'a> {
    /// A token was in some way unexpected
    UnexpectedToken(usize, Token<'a>, &'static str),
    ConversionError(ConversionError),
    ScalarError(ScalarError),
}

impl<'a> Display for SectionError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken(pos, tok, desc) => {
                write!(f, "token {:?} at {} is unexpected: {}", tok, pos, desc)
            }
            Self::ConversionError(err) => Display::fmt(err, f),
            Self::ScalarError(err) => Display::fmt(err, f),
        }
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
            _ => None,
        }
    }
}

/// A section of the save file.
/// It directly maps to a [SaveFileObject] and is the largest unit of data in the save file.
pub struct Section<'tape, 'data> {
    tape: Tokens<'tape, 'data>,
    offset: usize,
    length: usize,
    name: String,
}

impl<'tape, 'data> Section<'tape, 'data> {
    pub fn new(tape: Tokens<'tape, 'data>, name: String, offset: usize, length: usize) -> Self {
        Section {
            tape,
            name,
            offset,
            length,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn parse(&self) -> Result<SaveFileObject, SectionError> {
        let mut stack: Vec<(SaveFileObject, bool)> = vec![];
        let mut offset = self.offset;
        let mut key = false;
        match self.tape {
            Tokens::Text(text) => {
                while offset < self.length {
                    match &text[offset] {
                        TextToken::Array { end: _, mixed } => {
                            let arr = if offset == 0 {
                                GameObjectArray::from_name(self.name.clone())
                            } else if let Some(scalar) = &text[offset - 1].as_scalar() {
                                GameObjectArray::from_name(scalar.to_string())
                            } else {
                                GameObjectArray::new()
                            };
                            stack.push((SaveFileObject::Array(arr), *mixed));
                            key = false;
                        }
                        TextToken::End(_) => {
                            let new = stack.pop().unwrap().0;
                            let last = &mut stack.last_mut().unwrap().0;
                            match last {
                                SaveFileObject::Array(arr) => {
                                    arr.push(SaveFileValue::Object(new));
                                }
                                SaveFileObject::Map(map) => {
                                    map.insert(
                                        new.get_name().to_owned(),
                                        SaveFileValue::Object(new),
                                    );
                                }
                            }
                            key = false;
                        }
                        TextToken::Header(_) => {}
                        TextToken::Object { end: _, mixed } => {
                            let obj = if offset == 0 {
                                GameObjectMap::from_name(self.name.clone())
                            } else if let Some(scalar) = &text[offset - 1].as_scalar() {
                                GameObjectMap::from_name(scalar.to_string())
                            } else {
                                GameObjectMap::new()
                            };
                            stack.push((SaveFileObject::Map(obj), *mixed));
                            key = false;
                        }
                        TextToken::Quoted(string) | TextToken::Unquoted(string) => {
                            let (obj, mixed) = stack.last_mut().unwrap();
                            if *mixed {
                                if let TextToken::Operator(_) = &text[offset + 1] {
                                    // this is terrible, but probably least verbose
                                } else {
                                    match obj {
                                        SaveFileObject::Array(arr) => {
                                            arr.push(SaveFileValue::String(GameString::wrap(
                                                string.to_string(),
                                            )));
                                        }
                                        SaveFileObject::Map(map) => {
                                            // find the largest key in the map
                                            let mut largest_key = 0;
                                            for key in map.keys() {
                                                if let Ok(k) = key.parse::<i64>() {
                                                    if k > largest_key {
                                                        largest_key = k;
                                                    }
                                                }
                                            }
                                            map.insert(
                                                key.to_string(),
                                                SaveFileValue::String(GameString::wrap(
                                                    string.to_string(),
                                                )),
                                            );
                                        }
                                    }
                                }
                            } else {
                                // value of some kind
                                let val =
                                    SaveFileValue::String(GameString::wrap(string.to_string()));
                                match obj {
                                    SaveFileObject::Array(arr) => {
                                        arr.push(val);
                                    }
                                    SaveFileObject::Map(map) => {
                                        if key {
                                            if let Some(scalar) = &text[offset - 1].as_scalar() {
                                                map.insert(scalar.to_string(), val);
                                            } else {
                                                return Err(SectionError::UnexpectedToken(
                                                    offset,
                                                    Token::from_text(&text[offset - 1]),
                                                    "expected key is non scalar",
                                                ));
                                            }
                                            key = false;
                                        } else {
                                            key = true;
                                        }
                                    }
                                }
                            }
                        }
                        TextToken::Operator(op) => {
                            if *op != Operator::Equal {
                                return Err(SectionError::UnexpectedToken(
                                    offset,
                                    Token::from_text(&text[offset]),
                                    "encountered non = operator",
                                ));
                            }
                            let (obj, mixed) = stack.last_mut().unwrap();
                            if *mixed {
                                let key = if let Some(scalar) = &text[offset - 1].as_scalar() {
                                    scalar.to_string()
                                } else {
                                    return Err(SectionError::UnexpectedToken(
                                        offset,
                                        Token::from_text(&text[offset - 1]),
                                        "expected key is non scalar",
                                    ));
                                };
                                let val = if let Some(scalar) = &text[offset + 1].as_scalar() {
                                    SaveFileValue::String(GameString::wrap(scalar.to_string()))
                                } else {
                                    return Err(SectionError::UnexpectedToken(
                                        offset + 1,
                                        Token::from_text(&text[offset + 1]),
                                        "expected value is non scalar",
                                    ));
                                };
                                offset += 1;
                                match obj {
                                    SaveFileObject::Array(arr) => {
                                        let index: usize = key.parse()?;
                                        if index > arr.len() {
                                            arr.push(val); // MAYBE bad
                                        } else {
                                            arr.insert(index, val);
                                        }
                                    }
                                    SaveFileObject::Map(map) => {
                                        map.insert(key, val);
                                    }
                                }
                            } else {
                                return Err(SectionError::UnexpectedToken(
                                    offset,
                                    Token::from_text(&text[offset]),
                                    "operator in object",
                                ));
                            }
                        }
                        TextToken::MixedContainer => {}
                        TextToken::Parameter(_) | TextToken::UndefinedParameter(_) => {
                            return Err(SectionError::UnexpectedToken(
                                offset,
                                Token::from_text(&text[offset]),
                                "encountered Parameter*",
                            ));
                        }
                    }
                    offset += 1;
                }
            }
            Tokens::Binary(binary) => {
                fn add_key_value<'a, 'tok: 'a>(
                    stack: &mut Vec<(SaveFileObject, bool)>,
                    val: SaveFileValue,
                    key: &mut bool,
                    binary: &[BinaryToken<'tok>],
                    offset: usize,
                ) -> Result<(), SectionError<'a>> {
                    let (obj, _mixed) = stack.last_mut().unwrap();
                    match obj {
                        SaveFileObject::Array(arr) => {
                            arr.push(val);
                        }
                        SaveFileObject::Map(map) => {
                            if *key {
                                if let BinaryToken::Unquoted(scalar) = &binary[offset - 1] {
                                    map.insert(scalar.to_string(), val);
                                } else {
                                    return Err(SectionError::UnexpectedToken(
                                        offset,
                                        Token::from_binary(&binary[offset - 1]),
                                        "expected key",
                                    ));
                                }
                                *key = false;
                            } else {
                                *key = true;
                            }
                        }
                    }

                    return Ok(());
                }
                while offset < self.length {
                    match &binary[offset] {
                        BinaryToken::Array(_) => {
                            let arr = if let BinaryToken::Unquoted(scalar) = &binary[offset - 1] {
                                GameObjectArray::from_name(scalar.to_string())
                            } else {
                                GameObjectArray::new()
                            };
                            stack.push((SaveFileObject::Array(arr), false));
                        }
                        BinaryToken::Bool(b) => {
                            let val = SaveFileValue::Boolean(*b);
                            add_key_value(&mut stack, val, &mut key, binary, offset)?;
                        }
                        BinaryToken::End(_) => {
                            let new = stack.pop().unwrap().0;
                            let last = &mut stack.last_mut().unwrap().0;
                            match last {
                                SaveFileObject::Array(arr) => {
                                    arr.push(SaveFileValue::Object(new));
                                }
                                SaveFileObject::Map(map) => {
                                    map.insert(
                                        new.get_name().to_owned(),
                                        SaveFileValue::Object(new),
                                    );
                                }
                            }
                        }
                        BinaryToken::F32(val) => {
                            let val = SaveFileValue::Real(f32::from_le_bytes(*val) as f64);
                            add_key_value(&mut stack, val, &mut key, binary, offset)?;
                        }
                        BinaryToken::F64(val) => {
                            let val = SaveFileValue::Real(f64::from_le_bytes(*val));
                            add_key_value(&mut stack, val, &mut key, binary, offset)?;
                        }
                        BinaryToken::I32(val) => {
                            let val = SaveFileValue::Integer(*val as i64);
                            add_key_value(&mut stack, val, &mut key, binary, offset)?;
                        }
                        BinaryToken::I64(val) => {
                            let val = SaveFileValue::Integer(*val);
                            add_key_value(&mut stack, val, &mut key, binary, offset)?;
                        }
                        BinaryToken::U32(val) => {
                            let val = SaveFileValue::Integer(*val as i64);
                            add_key_value(&mut stack, val, &mut key, binary, offset)?;
                        }
                        BinaryToken::U64(val) => {
                            let val = SaveFileValue::Integer(*val as i64);
                            add_key_value(&mut stack, val, &mut key, binary, offset)?;
                        }
                        BinaryToken::MixedContainer | BinaryToken::Equal => {
                            // TODO fix alongside the rest of the binary parser
                            return Err(SectionError::UnexpectedToken(
                                offset,
                                Token::from_binary(&binary[offset]),
                                "unexpected MixedContainer",
                            ));
                        }
                        BinaryToken::Object(_) => {
                            let obj = if let BinaryToken::Unquoted(scalar) = &binary[offset - 1] {
                                GameObjectMap::from_name(scalar.to_string())
                            } else {
                                GameObjectMap::new()
                            };
                            stack.push((SaveFileObject::Map(obj), false));
                        }
                        BinaryToken::Quoted(string) | BinaryToken::Unquoted(string) => {
                            let val = SaveFileValue::String(GameString::wrap(string.to_string()));
                            add_key_value(&mut stack, val, &mut key, binary, offset)?;
                        }
                        BinaryToken::Rgb(rgb) => {
                            if let Some(a) = rgb.a {
                                let val = SaveFileValue::Integer(a as i64);
                                add_key_value(&mut stack, val, &mut key, binary, offset)?;
                            }
                            for el in [rgb.r, rgb.g, rgb.b] {
                                let val = SaveFileValue::Integer(el as i64);
                                add_key_value(&mut stack, val, &mut key, binary, offset)?;
                            }
                        }
                        BinaryToken::Token(_tok) => {
                            todo!()
                        }
                    }
                    offset += 1;
                }
            }
        }
        if stack.is_empty() {
            return Ok(SaveFileObject::Map(GameObjectMap::from_name(
                self.name.clone(),
            )));
        } else {
            return Ok(stack.pop().unwrap().0);
        }
    }
}

#[cfg(test)]
mod tests {

    use jomini::TextTape;

    use super::*;

    use super::super::types::Tape;

    #[test]
    fn test_empty() {
        let tape = Tape::Text(TextTape::from_slice(b"").unwrap());
        let tokens = tape.tokens();
        let section = Section::new(tokens, "empty".to_string(), 0, 0);
        let obj = section.parse().unwrap();
        assert_eq!(obj.get_name(), "empty");
        assert!(matches!(obj, SaveFileObject::Map(_)));
    }

    #[test]
    fn test_mixed_obj() {
        // MAYBE support this in the future, haven't seen it in the wild yet
        let tape = Tape::Text(TextTape::from_slice(b"test={a b 1=c 2={d=5}}").unwrap());
        let tokens = tape.tokens();
        let section = Section::new(tokens, "test".to_string(), 1, 14);
        let obj = section.parse();
        assert!(obj.is_err());
    }

    #[test]
    fn test_mixed() {
        let tape = Tape::Text(TextTape::from_slice(b"test={a b 1=c 2=d}").unwrap());
        let tokens = tape.tokens();
        let section = Section::new(tokens, "test".to_string(), 1, 11);
        let obj = section.parse().unwrap();
        assert_eq!(obj.get_name(), "test");
        if let SaveFileObject::Array(arr) = obj {
            assert_eq!(arr.len(), 4);
        } else {
            panic!("expected array");
        }
    }
}
