use super::{
    super::types::{Shared, Wrapper, WrapperMut},
    Section,
};
use std::{
    fmt::Debug,
    fs::File,
    io::{Cursor, Read},
    rc::Rc,
};
use zip::{read::ZipArchive, result::ZipError};

/// The header of an archive within a save file.
const ARCHIVE_HEADER: &[u8; 4] = b"PK\x03\x04";

/// An error that can occur when opening a save file.
pub enum SaveFileError {
    IoError(std::io::Error),
    ParseError(&'static str),
    DecompressionError(ZipError),
}

impl Debug for SaveFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveFileError::IoError(e) => write!(f, "IO error: {}", e),
            SaveFileError::ParseError(s) => write!(f, "Parse error: {}", s),
            SaveFileError::DecompressionError(e) => write!(f, "Decompression error: {}", e),
        }
    }
}

/// A struct that represents a ck3 save file.
/// This struct is an iterator that returns sections from the save file.
/// The save file is loaded once, and then parsed iteratively as needed.
/// If the parsing is done correctly the process should be O(n) speed and memory wise.
///
/// # Iterative behavior
///
/// The idea behind the iterative behavior is that the save file is parsed as needed.
/// Some sections are skipped, some are parsed into [SaveFileObject].
/// The choice is up to the user, after they are given the returned [Section] objects
/// where they can check the section name using [Section::get_name].
///
/// # Example
///
/// ```
/// let save = SaveFile::new("save.ck3");
/// for section in save{
///    println!("Section: {}", section.get_name());
/// }
pub struct SaveFile {
    /// The contents of the save file, shared between all sections
    contents: Rc<String>,
    /// A single shared byte offset for all sections inside of [SaveFile::contents]
    offset: Shared<usize>,
}

impl SaveFile {
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
        match File::open(filename) {
            Ok(mut file) => {
                let mut contents = vec![];
                if let Err(err) = file.read_to_end(&mut contents) {
                    return Err(SaveFileError::IoError(err));
                }
                let mut compressed = false;
                // find if ARCHIVE_HEADER is in the file
                for i in 0..contents.len() - ARCHIVE_HEADER.len() {
                    if contents[i..i + ARCHIVE_HEADER.len()] == *ARCHIVE_HEADER {
                        compressed = true;
                        break;
                    }
                }
                let contents = if compressed {
                    match ZipArchive::new(Cursor::new(contents)) {
                        Ok(mut archive) => match archive.by_index(0) {
                            Ok(mut gamestate) => {
                                if gamestate.is_dir() {
                                    return Err(SaveFileError::ParseError(
                                        "Save file is a directory",
                                    ));
                                }
                                let mut contents = String::new();
                                if let Err(err) = gamestate.read_to_string(&mut contents) {
                                    return Err(SaveFileError::IoError(err));
                                }
                                contents
                            }
                            Err(err) => {
                                return Err(SaveFileError::DecompressionError(err));
                            }
                        },
                        Err(err) => {
                            return Err(SaveFileError::DecompressionError(err));
                        }
                    }
                } else {
                    match String::from_utf8(contents) {
                        Ok(contents) => contents,
                        Err(_) => {
                            return Err(SaveFileError::ParseError("Save file is not valid UTF-8"));
                        }
                    }
                };
                return Ok(SaveFile {
                    contents: Rc::new(contents),
                    offset: Shared::wrap(0),
                });
            }
            Err(err) => {
                return Err(SaveFileError::IoError(err));
            }
        }
    }
}

impl SaveFile {
    /// Get the number of sections in the save file.
    pub fn len(&self) -> usize {
        let mut num = 0;
        let mut depth: u32 = 0;
        for c in self.contents.chars() {
            match c {
                '}' => {
                    depth -= 1;
                }
                '{' => {
                    if depth == 0 {
                        num += 1;
                    }
                    depth += 1;
                }
                _ => {}
            }
        }
        return num;
    }
}

impl Iterator for SaveFile {
    type Item = Result<Section, SaveFileError>;

    /// Get the next object in the save file
    /// If the file pointer has reached the end of the save file then it will return None.
    fn next(&mut self) -> Option<Self::Item> {
        let mut key = String::new();
        let off = self.offset.get_internal_mut();
        for c in self.contents[*off..].chars() {
            match c {
                '}' | '"' => {
                    return Some(Err(SaveFileError::ParseError(
                        "Unexpected character encountered in-between sections",
                    )));
                }
                '{' => {
                    break;
                }
                '\n' => {
                    key.clear();
                }
                ' ' | '\t' | '=' => {
                    continue;
                }
                _ => {
                    key.push(c);
                }
            }
        }
        if key.is_empty() {
            return None;
        }
        return Some(Ok(Section::new(
            key,
            self.contents.clone(),
            self.offset.clone(),
        )));
    }
}

#[cfg(test)]
mod tests {

    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    fn get_test_obj(input: &str) -> SaveFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(input.as_bytes()).unwrap();
        SaveFile::open(file.path().to_str().unwrap()).unwrap()
    }

    #[test]
    fn test_save_file() {
        let mut save_file = get_test_obj(
            "
            test={
                test2={
                    test3=1
                }
            }
        ",
        );
        let object = save_file.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.as_map().get_object_ref("test2").as_map();
        let test3 = test2.get_string_ref("test3");
        assert_eq!(*(test3), "1".to_string());
    }

    #[test]
    fn test_save_file_array() {
        let mut save_file = get_test_obj(
            "
            test={
                test2={
                    1
                    2
                    3
                }
                test3={ 1 2 3}
            }
        ",
        );
        let object = save_file.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.as_map().get_object_ref("test2");
        let test2_val = test2.as_array();
        assert_eq!(
            *(test2_val.get_index(0).unwrap().as_string()),
            "1".to_string()
        );
        assert_eq!(
            *(test2_val.get_index(1).unwrap().as_string()),
            "2".to_string()
        );
        assert_eq!(
            *(test2_val.get_index(2).unwrap().as_string()),
            "3".to_string()
        );
        let test3 = object.as_map().get_object_ref("test3");
        let test3_val = test3.as_array();
        assert_eq!(
            *(test3_val.get_index(0).unwrap().as_string()),
            "1".to_string()
        );
        assert_eq!(
            *(test3_val.get_index(1).unwrap().as_string()),
            "2".to_string()
        );
        assert_eq!(
            *(test3_val.get_index(2).unwrap().as_string()),
            "3".to_string()
        );
    }

    #[test]
    fn test_weird_syntax() {
        let mut save_file = get_test_obj(
            "
            test={
                test2={1=2
                    3=4}
                test3={1 2 
                    3}
                test4={1 2 3}
                test5=42
            }
        ",
        );
        let object = save_file.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.as_map().get_object_ref("test2").as_map();
        assert_eq!(*(test2.get_string_ref("1")), "2".to_string());
        assert_eq!(*(test2.get_string_ref("3")), "4".to_string());
    }

    #[test]
    fn test_array_syntax() {
        let mut save_file = get_test_obj(
            "
            test={
                test2={ 1 2 3 }
            }
        ",
        );
        let object = save_file.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.as_map().get_object_ref("test2").as_array();
        assert_eq!(*(test2.get_index(0).unwrap().as_string()), "1".to_string());
        assert_eq!(*(test2.get_index(1).unwrap().as_string()), "2".to_string());
        assert_eq!(*(test2.get_index(2).unwrap().as_string()), "3".to_string());
        assert_eq!(test2.len(), 3);
    }

    #[test]
    fn test_unnamed_obj() {
        let mut save_file = get_test_obj(
            "
        3623={
            name=\"dynn_Sao\"
            variables={
                data={ 
                        {
                            flag=\"ai_random_harm_cooldown\"
                            tick=7818
                            data={
                                type=boolean
                                identity=1
                            }
                        }
                        {
                            something_else=\"test\"
                        }
                    }
                }
            }
        }
        ",
        );
        let object = save_file.next().unwrap().unwrap().parse().unwrap();
        let variables = object.as_map().get_object_ref("variables").as_map();
        let data = variables.get_object_ref("data").as_array();
        assert_ne!(data.len(), 0)
    }

    #[test]
    fn test_example_1() {
        let mut save_file = get_test_obj("
        3623={
            name=\"dynn_Sao\"
            variables={
                data={ {
                        flag=\"ai_random_harm_cooldown\"
                        tick=7818
                        data={
                            type=boolean
                            identity=1
                        }
        
                    }
         }
            }
            found_date=750.1.1
            head_of_house=83939093
            dynasty=3623
            historical={ 4440 5398 6726 10021 33554966 50385988 77977 33583389 50381158 50425637 16880568 83939093 }
            motto={
                key=\"motto_with_x_I_seek_y\"
                variables={ {
                        key=\"1\"
                        value=\"motto_the_sword_word\"
                    }
         {
                        key=\"2\"
                        value=\"motto_bravery\"
                    }
         }
            }
            artifact_claims={ 83888519 }
        }");
        let object = save_file.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "3623".to_string());
        assert_eq!(
            *(object.as_map().get_string_ref("name")),
            "dynn_Sao".to_string()
        );
        let historical = object.as_map().get_object_ref("historical").as_array();
        assert_eq!(
            *(historical.get_index(0).unwrap().as_string()),
            "4440".to_string()
        );
        assert_eq!(
            *(historical.get_index(1).unwrap().as_string()),
            "5398".to_string()
        );
        assert_eq!(
            *(historical.get_index(2).unwrap().as_string()),
            "6726".to_string()
        );
        assert_eq!(
            *(historical.get_index(3).unwrap().as_string()),
            "10021".to_string()
        );
        assert_eq!(
            *(historical.get_index(4).unwrap().as_string()),
            "33554966".to_string()
        );
        assert_eq!(historical.len(), 12);
    }

    #[test]
    fn test_space() {
        let mut save_file = get_test_obj(
            "
        test = {
            test2 = {
                test3 = 1
            }
            test4 = { a b c}
        }
        ",
        );
        let object = save_file.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.as_map().get_object_ref("test2").as_map();
        let test3 = test2.get_string_ref("test3");
        assert_eq!(*(test3), "1".to_string());
        let test4 = object.as_map().get_object_ref("test4");
        let test4_val = test4.as_array();
        assert_eq!(
            *(test4_val.get_index(0).unwrap().as_string()),
            "a".to_string()
        );
        assert_eq!(
            *(test4_val.get_index(1).unwrap().as_string()),
            "b".to_string()
        );
        assert_eq!(
            *(test4_val.get_index(2).unwrap().as_string()),
            "c".to_string()
        );
    }

    #[test]
    fn test_landed() {
        let mut save_file = get_test_obj(
            "
        c_derby = {
            color = { 255 50 20 }

            cultural_names = {
                name_list_norwegian = cn_djuraby
                name_list_danish = cn_djuraby
                name_list_swedish = cn_djuraby
                name_list_norse = cn_djuraby
            }

            b_derby = {
                province = 1621

                color = { 255 89 89 }

                cultural_names = {
                    name_list_norwegian = cn_djuraby
                    name_list_danish = cn_djuraby
                    name_list_swedish = cn_djuraby
                    name_list_norse = cn_djuraby
                }
            }
            b_chesterfield = {
                province = 1622

                color = { 255 50 20 }
            }
            b_castleton = {
                province = 1623

                color = { 255 50 20 }
            }
        }
        ",
        );
        let object = save_file.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "c_derby".to_string());
        let b_derby = object.as_map().get_object_ref("b_derby").as_map();
        assert_eq!(*(b_derby.get_string_ref("province")), "1621".to_string());
        let b_chesterfield = object.as_map().get_object_ref("b_chesterfield").as_map();
        assert_eq!(
            *(b_chesterfield.get_string_ref("province")),
            "1622".to_string()
        );
        let b_castleton = object.as_map().get_object_ref("b_castleton").as_map();
        assert_eq!(
            *(b_castleton.get_string_ref("province")),
            "1623".to_string()
        );
    }

    #[test]
    fn test_invalid_line() {
        let mut save_file = get_test_obj(
            "
            some nonsense idk
            nonsense
            nonsense=idk
            test={
                test2={
                    test3=1
                }
            }
        ",
        );
        let object = save_file.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.as_map().get_object_ref("test2").as_map();
        let test3 = test2.get_string_ref("test3");
        assert_eq!(*(test3), "1".to_string());
    }

    #[test]
    fn test_empty() {
        let mut save_file = get_test_obj(
            "
            test={
            }
        ",
        );
        let object = save_file.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
    }

    #[test]
    fn test_arr_index() {
        let mut save_file = get_test_obj(
            "
            duration={ 2 0=7548 1=2096 }
        ",
        );
        let object = save_file.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "duration".to_string());
        assert_eq!(object.as_array().len(), 3);
    }

    #[test]
    fn test_multi_key() {
        let mut save_file = get_test_obj(
            "
        test={
            a=hello
            a=world
        }
        ",
        );
        let object = save_file.next().unwrap().unwrap().parse().unwrap();
        let arr = object.as_map().get_object_ref("a").as_array();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_invalid_syntax_1() {
        let mut save_file = get_test_obj(
            "
        test={
            a=hello
            b
        }
        ",
        );
        let object = save_file.next().unwrap().unwrap().parse();
        assert!(object.is_err())
    }

    #[test]
    fn test_invalid_syntax_2() {
        let mut save_file = get_test_obj(
            "
        test={
            b
            a=hello
        }
        ",
        );
        let object = save_file.next().unwrap().unwrap().parse();
        assert!(object.is_err())
    }
    #[test]
    fn test_invalid_syntax_3() {
        let mut save_file = get_test_obj(
            "
        b
        ",
        );
        let object = save_file.next();
        assert!(object.is_none())
    }
    #[test]
    fn test_invalid_syntax_4() {
        let mut save_file = get_test_obj(
            "
        b={
        ",
        );
        let object = save_file.next().unwrap().unwrap().parse();
        assert!(object.is_err())
    }
}
