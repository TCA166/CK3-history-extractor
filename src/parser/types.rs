use std::{
    error,
    fmt::{self, Debug},
};

use derive_more::{Display, From};
use jomini::{
    binary::{
        ReaderError as BinaryReaderError, Token as BinaryToken, TokenReader as BinaryTokenReader,
    },
    text::{ReaderError as TextReaderError, Token as TextToken, TokenReader as TextTokenReader},
};

/// An error that can occur when reading from a tape.
#[derive(Debug, From, Display)]
pub enum TapeError {
    Text(TextReaderError),
    Binary(BinaryReaderError),
}

impl error::Error for TapeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Text(err) => Some(err),
            Self::Binary(err) => Some(err),
        }
    }
}

/// An abstraction over the two [jomini] tape types: [jomini::TextTape] and [jomini::BinaryTape]
pub enum Tape<'a> {
    Text(TextTokenReader<&'a [u8]>),
    Binary(BinaryTokenReader<&'a [u8]>),
}

impl<'a> Tape<'a> {
    /// Skips all the tokens until it encounters the end of the current container.
    pub fn skip_container(&mut self) -> Result<(), TapeError> {
        match self {
            Self::Text(tape) => tape.skip_container()?,
            Self::Binary(tape) => tape.skip_container()?,
        }
        Ok(())
    }

    /// Gets the position in tape in bytes.
    pub fn position(&self) -> usize {
        match self {
            Self::Text(tape) => tape.position(),
            Self::Binary(tape) => tape.position(),
        }
    }
}

/* We only have this rather opaque abstraction, not a generalization of tokens
because a generalization would discard too much context. Most notably, the
information regarding whether a string was quoted or not. As such, we only
have this abstraction, used exclusively in error handling. */

/// An abstraction over [jomini] tokens: [jomini::TextToken] and [jomini::BinaryToken]
pub enum Token<'a> {
    Text(TextToken<'a>),
    Binary(BinaryToken<'a>),
}

impl<'a> From<TextToken<'a>> for Token<'a> {
    fn from(token: TextToken<'a>) -> Self {
        Self::Text(token)
    }
}

impl<'a> From<BinaryToken<'a>> for Token<'a> {
    fn from(token: BinaryToken<'a>) -> Self {
        Self::Binary(token)
    }
}

impl Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(token) => token.fmt(f),
            Self::Binary(token) => token.fmt(f),
        }
    }
}
