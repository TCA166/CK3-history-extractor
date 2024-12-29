use std::{
    error,
    fmt::{self, Debug, Display},
};

use jomini::{BinaryToken, TextToken};

use super::{
    section::Section,
    types::{Tape, Token, Tokens},
};

/// An error that occurred while reading sections from a tape.
#[derive(Debug)]
pub enum SectionReaderError<'err> {
    /// An unexpected token was encountered.
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
/// Yielded sections can (do not have to) be parsed using [Section::parse].
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
        let mut mixed = false; // mixed is local to the section, so the tokenizer sets it once every section
        match self.tape {
            Tokens::Text(text) => {
                while self.offset < self.length {
                    let tok = &text[self.offset];
                    match tok {
                        TextToken::Object { end, mixed: _ }
                        | TextToken::Array { end, mixed: _ } => {
                            // if we are in a mixed container, the key is two tokens back because = is inserted
                            let key_offset = if mixed { 2 } else { 1 };
                            if let TextToken::Unquoted(scalar) = &text[self.offset - key_offset] {
                                let key = scalar.to_string();
                                let section =
                                    Section::new(Tokens::Text(&text), key, self.offset, *end);
                                self.offset = *end + 1;
                                return Some(Ok(section));
                            } else {
                                return Some(Err(SectionReaderError::UnexpectedToken(
                                    self.offset - key_offset,
                                    Token::from_text(&text[self.offset - key_offset]),
                                    "non-scalar key encountered",
                                )));
                            }
                        }
                        TextToken::MixedContainer => {
                            mixed = true;
                            self.offset += 1;
                        }
                        TextToken::End(_)
                        | TextToken::Header(_)
                        | TextToken::UndefinedParameter(_)
                        | TextToken::Parameter(_) => {
                            return Some(Err(SectionReaderError::UnexpectedToken(
                                self.offset,
                                Token::from_text(tok),
                                "weird token in between sections",
                            )))
                        }
                        _ => {
                            // any token that may exist in the spaces between sections
                            // skip here, we aren't looking for un-sectioned key-value pairs
                            self.offset += 1;
                        }
                    }
                }
            }
            Tokens::Binary(binary) => {
                while self.offset < self.length {
                    let tok = &binary[self.offset];
                    match tok {
                        BinaryToken::Object(end) | BinaryToken::Array(end) => {
                            let key_offset = if mixed { 2 } else { 1 };
                            if let BinaryToken::Unquoted(scalar) = &binary[self.offset - key_offset]
                            {
                                let key = scalar.to_string();
                                let section = Section::new(
                                    Tokens::Binary(&binary),
                                    key,
                                    self.offset + 1,
                                    *end - self.offset,
                                );
                                self.offset = *end;
                                return Some(Ok(section));
                            } else {
                                return Some(Err(SectionReaderError::UnexpectedToken(
                                    self.offset,
                                    Token::from_binary(tok),
                                    "Non-ascii key encountered",
                                )));
                            }
                        }
                        BinaryToken::MixedContainer => {
                            mixed = true;
                            self.offset += 1;
                        }
                        BinaryToken::End(_) => {
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

#[cfg(test)]
mod tests {
    use jomini::TextTape;

    use super::*;

    #[test]
    fn test_empty() {
        let tape = Tape::Text(TextTape::from_slice(b"").unwrap());
        let mut reader = SectionReader::new(&tape);
        assert!(reader.next().is_none());
    }

    #[test]
    fn test_single_section() {
        let tape = Tape::Text(TextTape::from_slice(b"test={a=1}").unwrap());
        let mut reader = SectionReader::new(&tape);
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
    }

    #[test]
    fn test_single_section_messy() {
        let tape = Tape::Text(TextTape::from_slice(b" \t\r   test={a=1}   \t\r ").unwrap());
        let mut reader = SectionReader::new(&tape);
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        assert!(reader.next().is_none());
    }

    #[test]
    fn test_multiple_sections() {
        let tape = Tape::Text(TextTape::from_slice(b"test={a=1}test2={b=2}test3={c=3}").unwrap());
        let mut reader = SectionReader::new(&tape);
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test2");
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test3");
        assert!(reader.next().is_none());
    }

    #[test]
    fn test_non_ascii_key() {
        let tape = Tape::Text(TextTape::from_slice(b"test={\x80=1}").unwrap());
        let mut reader = SectionReader::new(&tape);
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
    }

    #[test]
    fn test_mixed() {
        let tape =
            Tape::Text(TextTape::from_slice(b"a\na=b\ntest={a=1}test2={b=2}test3={c=3}").unwrap());
        let mut reader = SectionReader::new(&tape);
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test2");
        let section = reader.next().unwrap().unwrap();
        assert_eq!(section.get_name(), "test3");
    }
}
