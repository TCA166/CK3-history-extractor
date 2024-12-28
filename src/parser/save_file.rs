use jomini::{self, BinaryTape, TextTape};
use std::{
    error,
    fmt::{self, Debug, Display},
    fs::File,
    io::{self, Cursor, Read},
    string::FromUtf8Error,
};
use zip::{read::ZipArchive, result::ZipError};

use super::types::Tape;

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
    /// Decoding bytes failed
    DecodingError(FromUtf8Error),
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
    fn from(value: FromUtf8Error) -> Self {
        Self::DecodingError(value)
    }
}

impl Display for SaveFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DecompressionError(err) => Display::fmt(err, f),
            Self::IoError(err) => Display::fmt(err, f),
            Self::DecodingError(err) => Display::fmt(err, f),
            Self::ParseError(err) => write!(f, "{}", err),
        }
    }
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
/// return an abstract [Tape] that can be used to read from the save file.
pub struct SaveFile {
    /// The contents of the save file, shared between all sections
    contents: Vec<u8>,
    binary: bool,
}

impl<'a> SaveFile {
    /// Open a save file.
    /// Internally uses [File::open] to open the file and then [SaveFile::read] to read the contents.
    pub fn open(filename: &str) -> Result<SaveFile, SaveFileError> {
        let mut file = File::open(filename)?;
        SaveFile::read(&mut file)
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
    pub fn read<F: Read>(file: &mut F) -> Result<SaveFile, SaveFileError> {
        let mut contents = vec![];
        let contents_size = file.read_to_end(&mut contents)?;
        if contents_size < ARCHIVE_HEADER.len() {
            return Err(SaveFileError::ParseError("Save file is too small"));
        }
        let mut compressed = false;
        let mut binary = false;
        // find if ARCHIVE_HEADER is in the file
        for i in 0..contents_size - ARCHIVE_HEADER.len() {
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
            if gamestate.read(&mut contents)? != gamestate_size {
                return Err(SaveFileError::ParseError("Failed to read the entire file"));
            }
            return Ok(SaveFile { contents, binary });
        } else {
            return Ok(SaveFile { contents, binary });
        }
    }

    /// Get the tape from the save file.
    pub fn tape(&'a self) -> Result<Tape<'a>, jomini::Error> {
        if self.binary {
            Ok(Tape::Text(TextTape::from_slice(&self.contents)?))
        } else {
            Ok(Tape::Binary(BinaryTape::from_slice(&self.contents)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use io::{Seek, SeekFrom};
    use zip::write::{SimpleFileOptions, ZipWriter};

    use super::*;

    // FIXME this somehow writes zip file that cannot be read back
    fn create_zipped_test_file(contents: &'static str) -> Cursor<Vec<u8>> {
        let file = Vec::new();
        let cur = Cursor::new(file);
        let mut zip = ZipWriter::new(cur);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
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
        let save = SaveFile::read(&mut file).unwrap();
        assert_eq!(save.contents, b"test");
    }

    #[test]
    fn test_compressed_open() {
        let mut file = create_zipped_test_file("test");
        let save = SaveFile::read(&mut file).unwrap();
        assert_eq!(save.contents, b"test");
    }
}
