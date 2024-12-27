use jomini::{self, BinaryTape, TextTape};
use std::{
    fmt::Debug,
    fs::File,
    io::{self, Cursor, Read},
    string::FromUtf8Error,
};
use zip::{read::ZipArchive, result::ZipError};

use super::types::{Tape, Tokens};

/// The header of an archive within a save file.
const ARCHIVE_HEADER: &[u8; 4] = b"PK\x03\x04";

const BINARY_HEADER: &[u8; 4] = b"U1\x01\x00";

/// An error that can occur when opening a save file.
/// Generally things that are the fault of the user, however unintentional those may be
#[derive(Debug)]
pub enum SaveFileError {
    /// Something went wrong with stdlib IO.
    IoError(io::Error),
    /// We found a problem
    ParseError(&'static str),
    /// Something went wrong with decompressing the save file.
    DecompressionError(ZipError),
    /// Jomini found a problem, on the tape level probably
    JominiError(jomini::Error),
}

impl From<ZipError> for SaveFileError {
    fn from(value: ZipError) -> Self {
        Self::DecompressionError(value)
    }
}

impl From<io::Error> for SaveFileError {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<FromUtf8Error> for SaveFileError {
    fn from(_: FromUtf8Error) -> Self {
        Self::ParseError("Failed to parse UTF8")
    }
}

impl From<jomini::Error> for SaveFileError {
    fn from(value: jomini::Error) -> Self {
        Self::JominiError(value)
    }
}

/// A struct that represents a ck3 save file.
pub struct SaveFile {
    /// The contents of the save file, shared between all sections
    contents: Vec<u8>,
    binary: bool,
}

impl<'a> SaveFile {
    /// Create a new SaveFile instance.
    /// The filename must be valid of course.
    ///
    /// # Compression
    ///
    /// The save file can be compressed using the zip format.
    /// Function will automatically detect if the save file is compressed and decompress it.
    ///
    /// # Returns
    ///
    /// A new SaveFile instance.
    /// It is an iterator that returns sections from the save file.
    pub fn open(filename: &str) -> Result<SaveFile, SaveFileError> {
        let mut file = File::open(filename)?;
        let mut contents = vec![];
        file.read_to_end(&mut contents)?;
        let mut compressed = false;
        let mut binary = false;
        // find if ARCHIVE_HEADER is in the file
        for i in 0..contents.len() - ARCHIVE_HEADER.len() {
            if contents[i..i + ARCHIVE_HEADER.len()] == *ARCHIVE_HEADER {
                compressed = true;
                break;
            } else if contents[i..i + BINARY_HEADER.len()] == *BINARY_HEADER {
                binary = true;
            }
        }
        if compressed {
            let mut archive = ZipArchive::new(Cursor::new(contents))?;
            let mut gamestate = archive.by_index(0)?;
            if gamestate.is_dir() {
                return Err(SaveFileError::ParseError("Save file is a directory"));
            }
            let mut contents = Vec::with_capacity(gamestate.size() as usize);
            gamestate.read(&mut contents)?;
            return Ok(SaveFile { contents, binary });
        } else {
            return Ok(SaveFile { contents, binary });
        }
    }

    pub fn tape(&'a self) -> Result<Tape<'a>, SaveFileError> {
        if self.binary {
            Ok(Tape::Text(TextTape::from_slice(&self.contents)?))
        } else {
            Ok(Tape::Binary(BinaryTape::from_slice(&self.contents)?))
        }
    }
}

//TODO add decoding tests
