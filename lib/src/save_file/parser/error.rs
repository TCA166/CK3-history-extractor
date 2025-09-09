use std::{error, fmt::Debug, num::ParseIntError};

use derive_more::{Display, From};
use jomini::common::DateError;

use super::{
    game_object::{ConversionError, KeyError, SaveObjectError},
    section::SectionError,
    section_reader::SectionReaderError,
};

/// An error that occurred somewhere within the broadly defined parsing process.
#[derive(Debug, From, Display)]
pub enum ParsingError {
    SectionError(SectionError),
    StructureError(SaveObjectError),
    ReaderError(String),
    JominiError(jomini::Error),
    DateError(DateError),
}

impl<'a, 'b: 'a> From<SectionReaderError<'b>> for ParsingError {
    fn from(value: SectionReaderError<'b>) -> Self {
        ParsingError::ReaderError(format!("{:?}", value))
    }
}

impl From<ConversionError> for ParsingError {
    fn from(value: ConversionError) -> Self {
        ParsingError::StructureError(value.into())
    }
}

impl From<ParseIntError> for ParsingError {
    fn from(value: ParseIntError) -> Self {
        ParsingError::StructureError(SaveObjectError::ConversionError(value.into()))
    }
}

impl From<KeyError> for ParsingError {
    fn from(value: KeyError) -> Self {
        ParsingError::StructureError(value.into())
    }
}

impl<'a> error::Error for ParsingError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::SectionError(err) => Some(err),
            Self::StructureError(err) => Some(err),
            Self::JominiError(err) => Some(err),
            Self::DateError(err) => Some(err),
            _ => None,
        }
    }
}
