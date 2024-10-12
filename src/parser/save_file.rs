use super::{
    super::types::{Shared, Wrapper, WrapperMut},
    game_object::{GameObjectArray, GameObjectMap, GameString, SaveFileObject, SaveFileValue},
};
use std::{io::Read, mem, rc::Rc, fs::File};
use zip::read::ZipArchive;

const ARCHIVE_HEADER: &[u8; 4] = b"PK\x03\x04";

/// A struct that represents a section in a ck3 save file.
/// Each section has a name, holds a reference to the contents of the save file and the current parsing offset.
/// This means that all sections of the save file share a single parsing state.
///
/// # Validity
///
/// A section is valid until it is converted into a [SaveFileObject] using [Section::parse] or skipped using [Section::skip].
/// After that, the section is invalidated and any further attempts to convert it will result in a panic.
/// This is done to prevent double parsing of the same section.
///
/// # Example
///
/// ```
/// let save = SaveFile::open("save.ck3");
/// let section = save.next();
/// let object = section.to_object().unwrap();
/// ```
pub struct Section {
    name: String,
    contents: Rc<String>,
    offset: Shared<usize>,
    valid: bool,
}

impl Section {
    /// Create a new section
    fn new(name: String, contents: Rc<String>, offset: Shared<usize>) -> Self {
        Section {
            name: name,
            contents: contents,
            offset: offset,
            valid: true,
        }
    }

    /// Get the name of the section
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Invalidate the section.
    /// Both [Section::parse] and [Section::skip] will panic if the section is invalid.
    fn invalidate(&mut self) {
        self.valid = false;
    }

    /// Parse the section into a [SaveFileObject].
    /// This is a rather costly process as it has to read the entire section contents and parse them.
    /// You can then make a choice if you want to parse the object or [Section::skip] it.
    ///
    /// # Panics
    ///
    /// Panics if the section is invalid, or a parsing error occurs.
    ///
    /// # Returns
    ///
    /// The parsed object. This can be a [GameObjectMap] or a [GameObjectArray].
    /// Note that empty objects are parsed as [GameObjectArray].
    /// Checking is object is empty via [SaveFileObject::is_empty] is a good idea before assuming it is a [GameObjectMap].
    pub fn parse(&mut self) -> SaveFileObject {
        if !self.valid {
            panic!("Section {} is invalid", self.name);
        }
        self.invalidate();
        let mut quotes = false;
        // storage
        let mut key = String::new(); // the key of the key value pair
        let mut val = String::new(); // the value of the key value pair
        let mut past_eq = false; // we use this flag to determine if we are parsing a key or a value
        let mut comment = false;
        let mut maybe_array = false; // if this is true, that means we encountered key =value
        let mut escape = false; // escape character toggle
        let mut name: Option<String> = None; // the name of the object we are parsing
        let mut depth: u32 = 0; // how deep we are in the object tree
                                //initialize the object stack
        let mut stack: Vec<SaveFileObject> = Vec::new();
        let mut off = self.offset.get_internal_mut();

        /// Add a character to the key or value
        fn add_normal(c: char, key: &mut String, val: &mut String, past_eq: &mut bool) {
            if *past_eq {
                val.push(c);
            } else {
                key.push(c);
            }
        }

        //initialize the key stack
        for (ind, c) in self.contents[*off..].char_indices() {
            match c {
                // we parse the byte
                '\r' => {} //what? BILL GATES! HE CAN'T KEEP GETTING AWAY WITH IT!!!!!!
                '{' => {
                    // we have a new object, we push a new object
                    if comment {
                        continue;
                    }
                    if quotes || escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        maybe_array = false;
                        depth += 1;
                        if name.is_some() {
                            if key.is_empty() {
                                stack.push(SaveFileObject::Array(GameObjectArray::from_name(
                                    name.take().unwrap(),
                                )));
                            } else {
                                stack.push(SaveFileObject::Map(GameObjectMap::from_name(
                                    name.take().unwrap(),
                                )));
                            }
                        }
                        // here's the thing, at this point we do not know if we are in an array or a map
                        // we will know that when we reach the first token
                        name = Some(mem::take(&mut key));
                        past_eq = false;
                    }
                }
                '}' => {
                    // we have reached the end of an object
                    if comment {
                        continue;
                    }
                    if quotes || escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        maybe_array = false;
                        // if there was an assignment, we insert the key value pair
                        if past_eq && !val.is_empty() {
                            if name.is_some() {
                                stack.push(SaveFileObject::Map(GameObjectMap::from_name(
                                    name.take().unwrap(),
                                )));
                            }
                            stack.last_mut().unwrap().as_map_mut().insert(
                                mem::take(&mut key),
                                SaveFileValue::String(GameString::wrap(mem::take(&mut val))),
                            );
                            past_eq = false;
                        }
                        // if there wasn't an assignment but we still gathered some data
                        else if !key.is_empty() {
                            if name.is_some() {
                                stack.push(SaveFileObject::Array(GameObjectArray::from_name(
                                    name.take().unwrap(),
                                )));
                            }
                            stack
                                .last_mut()
                                .unwrap()
                                .as_array_mut()
                                .push(SaveFileValue::String(GameString::wrap(mem::take(&mut key))));
                        } else if name.is_some() {
                            stack.push(SaveFileObject::Array(GameObjectArray::from_name(
                                name.take().unwrap(),
                            )));
                        }
                        // resolved the object, time to append it to the parent object
                        depth -= 1;
                        if depth > 0 {
                            // if we are still in an object, we pop the object and insert it into the parent object
                            let val = stack.pop().unwrap();
                            let parent = stack.last_mut().unwrap();
                            match parent {
                                SaveFileObject::Map(map) => {
                                    map.insert(
                                        val.get_name().to_owned(),
                                        SaveFileValue::Object(val),
                                    );
                                }
                                SaveFileObject::Array(array) => {
                                    array.push(SaveFileValue::Object(val));
                                }
                            }
                        } else if depth == 0 {
                            // we have reached the end of the object we are parsing, we return the object
                            *off += ind + 1;
                            break;
                        } else {
                            // sanity check
                            panic!("Depth is negative at {}", *off);
                        }
                    }
                }
                '"' => {
                    // TODO handle integers separately
                    // we have a quote, we toggle the quotes flag
                    if comment {
                        continue;
                    }
                    if escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        quotes = !quotes;
                    }
                }
                '\n' => {
                    // we have reached the end of a line, we check if we have a key value pair
                    if comment {
                        comment = false;
                        continue;
                    }
                    if quotes || escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        maybe_array = false;
                        if past_eq {
                            // we have a key value pair
                            past_eq = false; // we reset the past_eq flag
                            if name.is_some() {
                                stack.push(SaveFileObject::Map(GameObjectMap::from_name(
                                    name.take().unwrap(),
                                )));
                            } else if stack.is_empty() {
                                key.clear();
                                val.clear();
                                continue;
                            }
                            stack.last_mut().unwrap().as_map_mut().insert(
                                mem::take(&mut key),
                                SaveFileValue::String(GameString::wrap(mem::take(&mut val))),
                            );
                        } else if !key.is_empty() {
                            // we have just a key { \n key \n }
                            if name.is_some() {
                                stack.push(SaveFileObject::Array(GameObjectArray::from_name(
                                    name.take().unwrap(),
                                )));
                            } else if stack.is_empty() {
                                key.clear();
                                continue;
                            }
                            stack
                                .last_mut()
                                .unwrap()
                                .as_array_mut()
                                .push(SaveFileValue::String(GameString::wrap(mem::take(&mut key))));
                        }
                    }
                }
                ' ' | '\t' => {
                    //syntax sugar we ignore, most of the time, unless...
                    if comment {
                        continue;
                    }
                    if quotes || escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        // we are not in quotes, we check if we have a key value pair
                        // we are key=value <-here
                        if past_eq && !val.is_empty() {
                            if val == "rgb" || val == "hsv" {
                                //we are here color=rgb {}
                                val.clear();
                            } else {
                                // in case {something=else something=else} or { 1 0=1 1=2 2=3 }
                                if name.is_some() {
                                    stack.push(SaveFileObject::Map(GameObjectMap::from_name(
                                        name.take().unwrap(),
                                    )));
                                }
                                let last_frame = stack.last_mut().unwrap();
                                let val =
                                    SaveFileValue::String(GameString::wrap(mem::take(&mut val)));
                                match last_frame {
                                    SaveFileObject::Map(map) => {
                                        map.insert(mem::take(&mut key), val);
                                    }
                                    SaveFileObject::Array(array) => {
                                        let index = key.parse::<usize>().unwrap();
                                        key.clear();
                                        if index <= array.len() {
                                            array.insert(index, val);
                                        } else {
                                            // TODO technically this discards order, but I have yet to find a case where this is a problem
                                            array.push(val);
                                        }
                                    }
                                }
                                past_eq = false;
                            }
                        } else if !key.is_empty() && !past_eq {
                            // in case { something something something } OR key =value we want to preserve the spaces
                            maybe_array = true;
                        }
                    }
                }
                '=' => {
                    if comment {
                        continue;
                    }
                    maybe_array = false;
                    // if we have an assignment, we toggle adding from key to value
                    if quotes || escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        past_eq = true;
                    }
                }
                '#' => {
                    if quotes || escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        comment = true;
                    }
                }
                '\\' => {
                    if escape {
                        add_normal(c, &mut key, &mut val, &mut past_eq);
                        escape = false;
                    } else {
                        escape = true;
                    }
                }
                _ => {
                    //the main content we append to the key or value
                    if comment {
                        continue;
                    }
                    if maybe_array {
                        //we have a toggle that says that the last character was a space and key is not empty
                        if name.is_some() {
                            stack.push(SaveFileObject::Array(GameObjectArray::from_name(
                                name.take().unwrap(),
                            )));
                        } else if stack.is_empty() {
                            key.clear();
                            continue;
                        }
                        stack
                            .last_mut()
                            .unwrap()
                            .as_array_mut()
                            .push(SaveFileValue::String(GameString::wrap(mem::take(&mut key))));
                        maybe_array = false;
                    }
                    //we simply append the character to the key or value
                    add_normal(c, &mut key, &mut val, &mut past_eq);
                }
            }
        }
        let result = stack.pop().unwrap();
        return result;
    }

    /// Skip the current section.
    /// Adds the length of the section to the offset and returns.
    /// This is useful if you are not interested in the contents of the section.
    ///
    /// # Panics
    ///
    /// Panics if the section is invalid.
    pub fn skip(&mut self) {
        if !self.valid {
            panic!("Section {} is invalid", self.name);
        }
        self.invalidate();
        let mut depth = 0;
        let mut off = self.offset.get_internal_mut();
        for (ind, c) in self.contents[*off..].char_indices() {
            match c {
                '{' => {
                    depth += 1;
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        *off += ind + 1;
                        break;
                    }
                }
                _ => {}
            }
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
/// The choice is up to the user, after they are given the returned [Section] objects where they can check the section name using [Section::get_name].
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
    pub fn open(filename: &str) -> SaveFile {
        let mut file = File::open(filename).unwrap();
        let mut contents = vec![];
        if let Err(err) = file.read_to_end(&mut contents) {
            match err.kind() {
                std::io::ErrorKind::NotFound => {
                    panic!("Save file not found: {}", filename);
                }
                _ => {
                    panic!("Error opening save file: {}", err);
                }
            }
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
            let mut archive = ZipArchive::new(std::io::Cursor::new(contents)).unwrap();
            let mut gamestate = archive.by_index(0).unwrap();
            if gamestate.is_dir() {
                panic!("Gamestate is a directory");
            }
            let mut contents = String::new();
            gamestate.read_to_string(&mut contents).unwrap();
            contents
        } else {
            String::from_utf8(contents).unwrap()
        };
        SaveFile {
            contents: Rc::new(contents),
            offset: Shared::wrap(0),
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
    type Item = Section;

    /// Get the next object in the save file
    /// If the file pointer has reached the end of the save file then it will return None.
    fn next(&mut self) -> Option<Section> {
        let mut key = String::new();
        let off = self.offset.get_internal_mut();
        for c in self.contents[*off..].chars() {
            match c {
                '}' | '"' => {
                    panic!("Unexpected character at {}", *off);
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
        return Some(Section::new(
            key,
            self.contents.clone(),
            self.offset.clone(),
        ));
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
        SaveFile::open(file.path().to_str().unwrap())
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
        let object = save_file.next().unwrap().parse();
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
        let object = save_file.next().unwrap().parse();
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
        let object = save_file.next().unwrap().parse();
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
        let object = save_file.next().unwrap().parse();
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
        let object = save_file.next().unwrap().parse();
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
        let object = save_file.next().unwrap().parse();
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
        let object = save_file.next().unwrap().parse();
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
        let object = save_file.next().unwrap().parse();
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
        let object = save_file.next().unwrap().parse();
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
        let object = save_file.next().unwrap().parse();
        assert_eq!(object.get_name(), "test".to_string());
    }

    #[test]
    fn test_arr_index() {
        let mut save_file = get_test_obj(
            "
            duration={ 2 0=7548 1=2096 }
        ",
        );
        let object = save_file.next().unwrap().parse();
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
        let object = save_file.next().unwrap().parse();
        let arr = object.as_map().get_object_ref("a").as_array();
        assert_eq!(arr.len(), 2);
    }
}
