use derive_more::{Display, From};
use jomini::{
    self, binary::TokenReader as BinaryTokenReader, text::TokenReader as TextTokenReader,
};
use std::{
    error,
    fmt::Debug,
    fs::File,
    io::{self, Cursor, Read},
    path::Path,
    string::FromUtf8Error,
};
use zip::{read::ZipArchive, result::ZipError};

use super::types::Tape;

/// The header of an archive within a save file.
const ARCHIVE_HEADER: &[u8; 4] = b"PK\x03\x04";

const BINARY_HEADER: &[u8; 4] = b"U1\x01\x00";

/// An error that can occur when opening a save file.
/// Generally things that are the fault of the user, however unintentional those may be
#[derive(Debug, From, Display)]
pub enum SaveFileError {
    /// Something went wrong with stdlib IO.
    IoError(io::Error),
    /// We found a problem
    #[display("{}", _0)]
    ParseError(&'static str),
    /// Something went wrong with decompressing the save file.
    DecompressionError(ZipError),
    /// Decoding bytes failed
    DecodingError(FromUtf8Error),
}

impl error::Error for SaveFileError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::DecompressionError(err) => Some(err),
            Self::IoError(err) => Some(err),
            Self::DecodingError(err) => Some(err),
            Self::ParseError(_) => None,
        }
    }
}

/// A struct that represents a ck3 save file.
/// It is just a wrapper around the contents of the save file.
/// This is so that we can abstract away the compression, encoding and just
/// return an abstract object that can be used to read from the save file.
pub struct SaveFile {
    /// The contents of the save file, shared between all sections
    contents: Vec<u8>,
    binary: bool,
}

impl<'a> SaveFile {
    /// Open a save file.
    /// Internally uses [File::open] to open the file and then [SaveFile::read] to read the contents.
    pub fn open<P: AsRef<Path>>(filename: P) -> Result<SaveFile, SaveFileError> {
        let mut file = File::open(filename)?;
        let metadata = file.metadata()?;
        SaveFile::read(&mut file, Some(metadata.len() as usize))
    }

    /// Create a new SaveFile instance.
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
    pub fn read<F: Read>(
        file: &mut F,
        contents_size: Option<usize>,
    ) -> Result<SaveFile, SaveFileError> {
        let mut contents = if let Some(size) = contents_size {
            Vec::with_capacity(size)
        } else {
            Vec::new()
        };
        let read_size = file.read_to_end(&mut contents)?;
        if read_size < ARCHIVE_HEADER.len() {
            return Err(SaveFileError::ParseError("Save file is too small"));
        }
        let mut compressed = false;
        let mut binary = false;
        // find if ARCHIVE_HEADER is in the file
        for i in 0..read_size - ARCHIVE_HEADER.len() {
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
            if gamestate.name() != "gamestate" {
                return Err(SaveFileError::ParseError("Unexpected file name"));
            }
            let gamestate_size = gamestate.size() as usize;
            let mut contents = Vec::with_capacity(gamestate_size);
            if gamestate.read_to_end(&mut contents)? != gamestate_size {
                return Err(SaveFileError::ParseError("Failed to read the entire file"));
            }
            return Ok(SaveFile { contents, binary });
        } else {
            return Ok(SaveFile { contents, binary });
        }
    }

    /// Get the tape from the save file.
    pub fn tape(&'a self) -> Tape<'a> {
        if self.binary {
            Tape::Binary(BinaryTokenReader::new(&self.contents))
        } else {
            Tape::Text(TextTokenReader::new(&self.contents))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use io::{Seek, SeekFrom};
    use zip::write::{SimpleFileOptions, ZipWriter};

    use super::*;

    fn create_zipped_test_file(contents: &'static str) -> Cursor<Vec<u8>> {
        let file = Vec::new();
        let cur = Cursor::new(file);
        let mut zip = ZipWriter::new(cur);
        let options = SimpleFileOptions::default();
        zip.start_file("gamestate", options).unwrap();
        if zip.write(contents.as_bytes()).unwrap() != contents.len() {
            panic!("Failed to write the entire file");
        }
        let mut cur = zip.finish().unwrap();
        cur.seek(SeekFrom::Start(0)).unwrap();
        return cur;
    }

    #[test]
    fn test_open() {
        let mut file = Cursor::new(b"test");
        let save = SaveFile::read(&mut file, None).unwrap();
        assert_eq!(save.contents, b"test");
    }

    #[test]
    fn test_compressed_open() {
        let mut file = create_zipped_test_file("test");
        let save = SaveFile::read(&mut file, None).unwrap();
        assert_eq!(save.contents, b"test");
    }

    #[test]
    fn test_tape() {
        let mut file = Cursor::new(b"test=a");
        let save = SaveFile::read(&mut file, None).unwrap();
        let tape = save.tape();
        if let Tape::Binary(_) = tape {
            panic!("Expected text tape, got binary tape");
        }
    }
}
