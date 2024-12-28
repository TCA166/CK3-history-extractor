use std::{
    error,
    fmt::{self, Debug, Display},
};

use jomini::{BinaryToken, TextToken};

use super::{
    section::Section,
    types::{Tape, Token, Tokens},
};

#[derive(Debug)]
pub enum SectionReaderError<'err> {
    UnexpectedToken(usize, Token<'err>, &'static str),
}

impl<'err> Display for SectionReaderError<'err> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken(pos, tok, desc) => {
                write!(
                    f,
                    "reader encountered an unexpected token {:?} at {}: {}",
                    tok, pos, desc
                )
            }
        }
    }
}

impl<'err> error::Error for SectionReaderError<'err> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

/// A reader for sections in a tape.
/// This reader will iterate over the tape and return sections, which are
/// largest [Objects](super::SaveFileObject) in the save file.
pub struct SectionReader<'tape, 'data> {
    /// The tape to read from.
    tape: Tokens<'tape, 'data>,
    /// The current offset in the tape.
    offset: usize,
    /// The length of the tape.
    length: usize,
}

impl<'tape, 'data> SectionReader<'tape, 'data> {
    /// Create a new section reader from a tape.
    pub fn new(tape: &'data Tape<'data>) -> Self {
        let tape = tape.tokens();
        let length = tape.len();
        SectionReader {
            tape,
            offset: 0,
            length,
        }
    }

    /// Get the number of sections in the tape.
    pub fn len(&self) -> u32 {
        let mut offset = 0;
        let mut result = 0;
        match self.tape {
            Tokens::Text(text) => {
                while offset < self.length {
                    match &text[offset] {
                        TextToken::Object { end, mixed: _ }
                        | TextToken::Array { end, mixed: _ } => {
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
    type Item = Result<Section<'tape, 'data>, SectionReaderError<'data>>;

    fn next(&mut self) -> Option<Result<Section<'tape, 'data>, SectionReaderError<'data>>> {
        match self.tape {
            Tokens::Text(text) => {
                while self.offset < self.length {
                    let tok = &text[self.offset];
                    match tok {
                        TextToken::Object { end, mixed: _ }
                        | TextToken::Array { end, mixed: _ } => {
                            if let Some(scalar) = text[self.offset - 1].as_scalar() {
                                if !scalar.is_ascii() {
                                    return Some(Err(SectionReaderError::UnexpectedToken(
                                        self.offset,
                                        Token::from_text(tok),
                                        "non-ascii key encountered",
                                    )));
                                }
                                let key = scalar.to_string();
                                let section =
                                    Section::new(Tokens::Text(&text), key, self.offset, *end);
                                self.offset += *end;
                                return Some(Ok(section));
                            } else {
                                return Some(Err(SectionReaderError::UnexpectedToken(
                                    self.offset,
                                    Token::from_text(tok),
                                    "non-scalar key encountered",
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
                                self.offset,
                                Token::from_text(tok),
                                "weird token in between sections",
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
                                        self.offset,
                                        Token::from_binary(tok),
                                        "Non-ascii key encountered",
                                    )));
                                }
                            }
                        }
                        BinaryToken::End(_) | BinaryToken::MixedContainer => {
                            return Some(Err(SectionReaderError::UnexpectedToken(
                                self.offset,
                                Token::from_binary(tok),
                                "Weird token in between sections",
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
