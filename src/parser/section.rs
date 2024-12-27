use jomini::{text::Operator, BinaryToken, ScalarError, TextToken};

use super::{
    super::types::Wrapper, game_object::ConversionError, types::Tokens, GameObjectArray,
    GameObjectMap, GameString, SaveFileObject, SaveFileValue,
};

/// An error that occured while processing a specific section
#[derive(Debug)]
pub enum SectionError {
    /// A token was in some way unexpected
    UnexpectedToken(&'static str, usize),
    ConversionError(ConversionError),
    ScalarError(ScalarError),
}

impl From<ConversionError> for SectionError {
    fn from(value: ConversionError) -> Self {
        Self::ConversionError(value)
    }
}

impl From<ScalarError> for SectionError {
    fn from(value: ScalarError) -> Self {
        Self::ScalarError(value)
    }
}

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
        let mut stack = vec![];
        let mut offset = self.offset;
        let mut key = false;
        match self.tape {
            Tokens::Text(text) => {
                while offset < self.length {
                    match &text[offset] {
                        TextToken::Array { .. } => {
                            let arr = if offset == 0 {
                                GameObjectArray::from_name(self.name.clone())
                            } else if let Some(scalar) = &text[offset - 1].as_scalar() {
                                GameObjectArray::from_name(scalar.to_string())
                            } else {
                                GameObjectArray::new()
                            };
                            stack.push(SaveFileObject::Array(arr));
                            key = false;
                        }
                        TextToken::End(_) => {
                            let new = stack.pop().unwrap();
                            let last = stack.last_mut().unwrap();
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
                        TextToken::Object { .. } => {
                            let obj = if offset == 0 {
                                GameObjectMap::from_name(self.name.clone())
                            } else if let Some(scalar) = &text[offset - 1].as_scalar() {
                                GameObjectMap::from_name(scalar.to_string())
                            } else {
                                GameObjectMap::new()
                            };
                            stack.push(SaveFileObject::Map(obj));
                            key = false;
                        }
                        TextToken::Quoted(string) | TextToken::Unquoted(string) => {
                            // value of some kind
                            let val = SaveFileValue::String(GameString::wrap(string.to_string()));
                            let obj = stack.last_mut().unwrap();
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
                                                "Invalid key",
                                                offset,
                                            ));
                                        }
                                        key = false;
                                    } else {
                                        key = true;
                                    }
                                }
                            }
                        }
                        TextToken::Operator(op) => {
                            if *op != Operator::Equal {
                                return Err(SectionError::UnexpectedToken(
                                    "Unexpected token",
                                    offset,
                                ));
                            }
                            let obj = stack.last_mut().unwrap();
                            match obj {
                                SaveFileObject::Array(arr) => {
                                    let index = arr.pop().unwrap().as_integer()?;
                                    if let Some(value) = &text[offset + 1].as_scalar() {
                                        arr.insert(
                                            index as usize,
                                            SaveFileValue::String(GameString::wrap(
                                                value.to_string(),
                                            )),
                                        );
                                        offset += 1;
                                    } else {
                                        return Err(SectionError::UnexpectedToken(
                                            "Unexpected token",
                                            offset + 1,
                                        ));
                                    }
                                }
                                _ => {
                                    return Err(SectionError::UnexpectedToken(
                                        "Unexpected token",
                                        offset,
                                    ));
                                }
                            }
                        }
                        TextToken::MixedContainer => {
                            let obj = stack.last().unwrap();
                            match obj {
                                SaveFileObject::Map(_) => {
                                    return Err(SectionError::UnexpectedToken(
                                        "Unexpected token",
                                        offset,
                                    ));
                                }
                                _ => {}
                            }
                        }
                        TextToken::Parameter(_) | TextToken::UndefinedParameter(_) => {
                            return Err(SectionError::UnexpectedToken("Unexpected token", offset));
                        }
                    }
                    offset += 1;
                }
            }
            Tokens::Binary(binary) => {
                fn add_key_value(
                    stack: &mut Vec<SaveFileObject>,
                    val: SaveFileValue,
                    key: &mut bool,
                    binary: &[BinaryToken<'_>],
                    offset: usize,
                ) -> Result<(), SectionError> {
                    let obj = stack.last_mut().unwrap();
                    match obj {
                        SaveFileObject::Array(arr) => {
                            arr.push(val);
                        }
                        SaveFileObject::Map(map) => {
                            if *key {
                                match &binary[offset - 1] {
                                    BinaryToken::Unquoted(scalar) => {
                                        map.insert(scalar.to_string(), val);
                                    }
                                    _ => {
                                        return Err(SectionError::UnexpectedToken(
                                            "Invalid key",
                                            offset,
                                        ));
                                    }
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
                            let arr = match &binary[offset - 1] {
                                BinaryToken::Unquoted(scalar) => {
                                    GameObjectArray::from_name(scalar.to_string())
                                }
                                _ => GameObjectArray::new(),
                            };
                            stack.push(SaveFileObject::Array(arr));
                        }
                        BinaryToken::Bool(b) => {
                            let val = SaveFileValue::Boolean(*b);
                            add_key_value(&mut stack, val, &mut key, binary, offset)?;
                        }
                        BinaryToken::End(_) => {
                            let new = stack.pop().unwrap();
                            let last = stack.last_mut().unwrap();
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
                            return Err(SectionError::UnexpectedToken(
                                "Encountered mixed container",
                                offset,
                            ));
                        }
                        BinaryToken::Object(_) => {
                            let obj = match &binary[offset - 1] {
                                BinaryToken::Unquoted(scalar) => {
                                    GameObjectMap::from_name(scalar.to_string())
                                }
                                _ => GameObjectMap::new(),
                            };
                            stack.push(SaveFileObject::Map(obj));
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
                        BinaryToken::Token(tok) => {
                            todo!()
                        }
                    }
                    offset += 1;
                }
            }
        }
        return Ok(stack.pop().unwrap());
    }
}

// TODO add tests covering section parsing
