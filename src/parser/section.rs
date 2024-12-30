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
    /// An error occured while converting a value
    ConversionError(ConversionError),
    /// An error occured while parsing a scalar
    ScalarError(ScalarError),
    /// An unknown token was encountered
    UnknownToken(u16),
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

fn token_resolver(token: &u16) -> Result<&'static str, SectionError> {
    match token {
        // TODO
        _ => Err(SectionError::UnknownToken(*token)),
    }
}

/// A section of the save file.
/// It directly maps to a [SaveFileObject] and is the largest unit of data in the save file.
pub struct Section<'tape, 'data> {
    tape: Tokens<'tape, 'data>,
    offset: usize,
    end: usize,
    name: String,
}

impl<'tape, 'data> Section<'tape, 'data> {
    /// Create a new section from a tape.
    /// The section will be named `name` and will start at `offset` and end at `end`.
    /// The first token of the section (pointed at by `offset`) is expected to an object or array token.
    /// The end token is not included in the section.
    pub fn new(tape: Tokens<'tape, 'data>, name: String, offset: usize, end: usize) -> Self {
        Section {
            tape,
            name,
            offset,
            end,
        }
    }

    /// Get the name of the section.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /* Looking at this parser, you can quite easily see an opportunity for an
    abstraction based on an enum. This would allow us to have a single parser
    that can handle both text and binary tokens. The problem with that approach
    is that we would have to go even lower level, and for example.: manually
    determine which objects are arrays and which are maps. This I believe would
    be a step too far, maybe in the future if performance starts being an issue
    now it's just not worth it. */

    /// Parse the section into a [SaveFileObject].
    pub fn parse(&self) -> Result<SaveFileObject, SectionError> {
        let mut stack: Vec<(SaveFileObject, bool)> = vec![];
        let mut offset = self.offset;
        let mut key = false;
        match self.tape {
            Tokens::Text(text) => {
                while offset < self.end {
                    match &text[offset] {
                        // we found an array, add it to the stack
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
                        // we found an object, add it to the stack
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
                        // we found an end to something, pop the stack and add the object to the parent
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
                        // we found a value, add it to the parent
                        TextToken::Quoted(string) | TextToken::Unquoted(string) => {
                            let val = SaveFileValue::String(GameString::wrap(string.to_string()));
                            let (obj, mixed) = stack.last_mut().unwrap();
                            if *mixed {
                                if let TextToken::Operator(_) = &text[offset - 1] {
                                    // we are in the value part of the assignment
                                    if let TextToken::Unquoted(scalar) = &text[offset - 2] {
                                        match obj {
                                            SaveFileObject::Array(arr) => {
                                                let index = scalar.to_u64()? as usize;
                                                if index >= arr.len() {
                                                    arr.push(val);
                                                } else {
                                                    arr.insert(index, val);
                                                }
                                            }
                                            SaveFileObject::Map(map) => {
                                                map.insert(scalar.to_string(), val);
                                            }
                                        }
                                    } else {
                                        return Err(SectionError::UnexpectedToken(
                                            offset,
                                            Token::from_text(&text[offset - 2]),
                                            "expected key is non scalar",
                                        ));
                                    }
                                } else if let TextToken::Operator(_) = &text[offset + 1] {
                                    // we are in the key part of the assingment
                                } else {
                                    // we are in an array assignment
                                    match obj {
                                        // if array we push
                                        SaveFileObject::Array(arr) => {
                                            arr.push(val);
                                        }
                                        // else we find the largest numerical key and insert
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
                                            map.insert(key.to_string(), val);
                                        }
                                    }
                                }
                            } else {
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
                        // we found an operator, do a sanity check and skip
                        TextToken::Operator(op) => {
                            if *op != Operator::Equal {
                                return Err(SectionError::UnexpectedToken(
                                    offset,
                                    Token::from_text(&text[offset]),
                                    "encountered non = operator",
                                ));
                            }
                        }
                        // we don't care about these
                        TextToken::MixedContainer | TextToken::Header(_) => {}
                        // these are eu4 exclusive, something went wrong
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
                /// Convenience function to avoid duplicate code
                fn add_key_value<'a, 'tok: 'a>(
                    stack: &mut Vec<(SaveFileObject, bool)>,
                    val: SaveFileValue,
                    key: &mut bool,
                    binary: &'a [BinaryToken<'tok>],
                    offset: usize,
                ) -> Result<(), SectionError<'a>> {
                    let (obj, mixed) = stack.last_mut().unwrap();
                    if *mixed {
                        if let BinaryToken::Equal = &binary[offset - 1] {
                            if let BinaryToken::Token(token) = &binary[offset - 2] {
                                let key = token_resolver(token)?;
                                match obj {
                                    SaveFileObject::Array(arr) => {
                                        let index = key.parse()?;
                                        if index >= arr.len() {
                                            arr.push(val);
                                        } else {
                                            arr.insert(index, val);
                                        }
                                    }
                                    SaveFileObject::Map(map) => {
                                        map.insert(key.to_string(), val);
                                    }
                                }
                            } else {
                                return Err(SectionError::UnexpectedToken(
                                    offset,
                                    Token::from_binary(&binary[offset - 1]),
                                    "expected key",
                                ));
                            }
                        } else if let BinaryToken::Equal = &binary[offset + 1] {
                            // we are a key in a key-value assignment
                            // skip
                        } else {
                            // we are in an array
                            match obj {
                                SaveFileObject::Array(arr) => {
                                    arr.push(val);
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
                                    map.insert(largest_key.to_string(), val);
                                }
                            }
                        }
                    } else {
                        match obj {
                            SaveFileObject::Array(arr) => {
                                arr.push(val);
                            }
                            SaveFileObject::Map(map) => {
                                if *key {
                                    if let BinaryToken::Token(token) = &binary[offset - 1] {
                                        let key = token_resolver(token)?;
                                        map.insert(key.to_string(), val);
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
                    }
                    return Ok(());
                }
                while offset < self.end {
                    match &binary[offset] {
                        BinaryToken::Array(_) => {
                            let arr = if offset == 0 {
                                GameObjectArray::from_name(self.name.clone())
                            } else if let BinaryToken::Token(token) = &binary[offset - 1] {
                                let key = token_resolver(token)?;
                                GameObjectArray::from_name(key.to_string())
                            } else {
                                GameObjectArray::new()
                            };
                            stack.push((SaveFileObject::Array(arr), false));
                            key = false;
                        }
                        BinaryToken::Object(_) => {
                            let obj = if offset == 0 {
                                GameObjectMap::from_name(self.name.clone())
                            } else if let BinaryToken::Token(token) = &binary[offset - 1] {
                                let key = token_resolver(token)?;
                                GameObjectMap::from_name(key.to_string())
                            } else {
                                GameObjectMap::new()
                            };
                            stack.push((SaveFileObject::Map(obj), false));
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
                            key = false;
                        }
                        BinaryToken::Bool(b) => {
                            let val = SaveFileValue::Boolean(*b);
                            add_key_value(&mut stack, val, &mut key, binary, offset)?;
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
                        BinaryToken::Quoted(string) | BinaryToken::Unquoted(string) => {
                            let val = SaveFileValue::String(GameString::wrap(string.to_string()));
                            add_key_value(&mut stack, val, &mut key, binary, offset)?;
                        }
                        BinaryToken::MixedContainer => {
                            stack.last_mut().unwrap().1 = true;
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
                        BinaryToken::Equal | BinaryToken::Token(_) => {}
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

impl Debug for Section<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Section")
            .field("name", &self.name)
            .field("offset", &self.offset)
            .field("end", &self.end)
            .finish()
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
        let tape = Tape::Text(TextTape::from_slice(b"test={a b 1=c 2={d=5}}").unwrap());
        let tokens = tape.tokens();
        let section = Section::new(tokens, "test".to_string(), 1, 14);
        let obj = section.parse();
        assert!(obj.is_ok());
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
