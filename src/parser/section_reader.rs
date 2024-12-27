use std::fmt::Debug;

use jomini::{BinaryToken, TextToken};

use super::{
    section::Section,
    types::{Tape, Tokens},
};

#[derive(Debug)]
pub enum SectionReaderError {
    UnexpectedToken(&'static str, usize),
}

pub struct SectionReader<'tape, 'data> {
    tape: Tokens<'tape, 'data>,
    offset: usize,
    length: usize,
}

impl<'tape, 'data> SectionReader<'tape, 'data> {
    pub fn new(tape: &'data Tape<'data>) -> Self {
        let tape = tape.tokens();
        let length = tape.len();
        SectionReader {
            tape,
            offset: 0,
            length,
        }
    }

    pub fn len(&self) -> u32 {
        let mut offset = 0;
        let mut result = 0;
        match self.tape {
            Tokens::Text(text) => {
                while offset < self.length {
                    match &text[offset] {
                        TextToken::Object { end, mixed } | TextToken::Array { end, mixed } => {
                            offset = *end;
                            result += 1;
                        }
                        _ => {
                            offset += 1;
                        }
                    }
                }
            }
            Tokens::Binary(binary) => {
                while offset < self.length {
                    match &binary[offset] {
                        BinaryToken::Object(end) | BinaryToken::Array(end) => {
                            offset = *end;
                            result += 1;
                        }
                        _ => {
                            offset += 1;
                        }
                    }
                }
            }
        }
        return result;
    }
}

impl<'tape, 'data> Iterator for SectionReader<'tape, 'data> {
    type Item = Result<Section<'tape, 'data>, SectionReaderError>;

    fn next(&mut self) -> Option<Result<Section<'tape, 'data>, SectionReaderError>> {
        match self.tape {
            Tokens::Text(text) => {
                while self.offset < self.length {
                    let tok = &text[self.offset];
                    match tok {
                        TextToken::Object { end, mixed } | TextToken::Array { end, mixed } => {
                            if let Some(scalar) = text[self.offset - 1].as_scalar() {
                                if !scalar.is_ascii() {
                                    return Some(Err(SectionReaderError::UnexpectedToken(
                                        "Non-ascii key encountered",
                                        self.offset,
                                    )));
                                }
                                let key = scalar.to_string();
                                let section =
                                    Section::new(Tokens::Text(&text), key, self.offset, *end);
                                self.offset += *end;
                                return Some(Ok(section));
                            } else {
                                return Some(Err(SectionReaderError::UnexpectedToken(
                                    "Non-scalar key encountered",
                                    self.offset,
                                )));
                            }
                        }
                        TextToken::Quoted(_) | TextToken::Unquoted(_) | TextToken::Operator(_) => {
                            // any token that may exist in the spaces between sections
                            // skip here, we aren't looking for un-sectioned key-value pairs
                            self.offset += 1;
                        }
                        _ => {
                            return Some(Err(SectionReaderError::UnexpectedToken(
                                "Unexpected token",
                                self.offset,
                            )))
                        }
                    }
                }
            }
            Tokens::Binary(binary) => {
                while self.offset < self.length {
                    let tok = &binary[self.offset];
                    match tok {
                        BinaryToken::Object(end) | BinaryToken::Array(end) => {
                            let key_token = &binary[self.offset - 1];
                            match key_token {
                                BinaryToken::Unquoted(scalar) => {
                                    let key = scalar.to_string();
                                    let section = Section::new(
                                        Tokens::Binary(&binary),
                                        key,
                                        self.offset + 1,
                                        *end - self.offset,
                                    );
                                    self.offset = *end;
                                    return Some(Ok(section));
                                }
                                _ => {
                                    return Some(Err(SectionReaderError::UnexpectedToken(
                                        "Non-ascii key encountered",
                                        self.offset,
                                    )));
                                }
                            }
                        }
                        BinaryToken::End(_) | BinaryToken::MixedContainer => {
                            return Some(Err(SectionReaderError::UnexpectedToken(
                                "Unexpected token",
                                self.offset,
                            )));
                        }
                        _ => {
                            self.offset += 1;
                        }
                    }
                }
            }
        }
        return None;
    }
}

// TODO add tests covering section initialization and iteration
// what happens when the tape is empty?
// what happens when the tape has a single section?
// what happens when the tape has multiple sections?
// what happens when the tape has a section with a non-ascii key?
// what happens when the tape has a section with a non-scalar key?
// what happens when the tape has an unclosed section?
