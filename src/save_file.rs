use std::{io::Read, mem, rc::Rc};
use crate::{game_object::{GameObject, SaveFileValue}, types::{Shared, Wrapper, WrapperMut}};
use zip::read::ZipArchive;

/// A struct that represents a section in a ck3 save file.
/// Each section has a name, holds a reference to the contents of the save file and the current parsing offset.
/// This means that all sections of the save file share a single parsing state.
/// 
/// # Validity
/// 
/// A section is valid until it is converted into a [GameObject] using [Section::to_object] or skipped using [Section::skip].
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
pub struct Section{
    name: String,
    contents: Rc<String>,
    offset: Shared<usize>,
    valid: bool
}

impl Section{
    /// Create a new section
    fn new(name: String, contents: Rc<String>, offset: Shared<usize>) -> Self{
        Section{
            name: name,
            contents: contents,
            offset: offset,
            valid: true
        }
    }

    /// Get the name of the section
    pub fn get_name(&self) -> &str{
        &self.name
    }

    /// Invalidate the section.
    /// Both [Section::to_object] and [Section::skip] will panic if the section is invalid.
    fn invalidate(&mut self){
        self.valid = false;
    }

    /// Convert the section to a GameObject.
    /// This is a rather costly process as it has to read the entire section contents and parse them.
    /// You can then make a choice if you want to parse the object or [Section::skip] it.
    /// 
    /// # Panics
    /// 
    /// Panics if the section is invalid, or a parsing error occurs.
    pub fn to_object(&mut self) -> GameObject{
        if !self.valid{
            panic!("Section {} is invalid", self.name);
        }
        self.invalidate();
        let mut quotes = false;
        // storage
        let mut key = String::new();
        let mut val = String::new();
        let mut past_eq = false; // we use this flag to determine if we are parsing a key or a value
        let mut comment = false;
        let mut maybe_array = false;
        let mut depth = 0; // how deep we are in the object tree
        let mut object:GameObject = GameObject::new();
        //initialize the object stack
        let mut stack: Vec<GameObject> = Vec::new();
        stack.push(object);
        let mut off = self.offset.get_internal_mut();
        //initialize the key stack
        for (ind, c) in self.contents[*off..].char_indices() { 
            match c{ // we parse the byte
                '\r' => {} //what? BILL GATES! HE CAN'T KEEP GETTING AWAY WITH IT!!!!!!
                '{' => { // we have a new object, we push a new hashmap to the stack
                    if comment{
                        continue;
                    }
                    maybe_array = false;
                    depth += 1;
                    stack.push(GameObject::from_name(mem::take(&mut key)));
                    past_eq = false;
                }
                '}' => { // we have reached the end of an object
                    if comment{
                        continue;
                    }
                    maybe_array = false;
                    // if there was an assignment, we insert the key value pair
                    if past_eq && !val.is_empty() {
                        stack.last_mut().unwrap().insert(mem::take(&mut key), SaveFileValue::String(Rc::new(mem::take(&mut val))));
                        past_eq = false;
                    }
                    // if there wasn't an assignment but we still gathered some data
                    else if !key.is_empty() {
                        stack.last_mut().unwrap().push(SaveFileValue::String(Rc::new(mem::take(&mut key))));
                    }
                    depth -= 1;
                    if depth > 0 { // if we are still in an object, we pop the object and insert it into the parent object
                        let inner = stack.pop().unwrap();
                        let name = inner.get_name().to_string();
                        let val = SaveFileValue::Object(inner);
                        if name.is_empty(){ //check if unnamed node, implies we are dealing with an array of unnamed objects
                            stack.last_mut().unwrap().push(val);
                        }
                        else{
                            stack.last_mut().unwrap().insert(name, val);
                        }
                    }
                    else if depth == 0{ // we have reached the end of the object we are parsing, we return the object
                        *off += ind + 1;
                        break;
                    }
                    else { // sanity check
                        panic!("Depth is negative at {}", *off);
                    }
                }
                '"' => { // we have a quote, we toggle the quotes flag
                    if comment{
                        continue;
                    }
                    quotes = !quotes;
                }
                '\n' => { // we have reached the end of a line, we check if we have a key value pair
                    if comment{
                        comment = false;
                    }
                    maybe_array = false;
                    if past_eq { // we have a key value pair
                        stack.last_mut().unwrap().insert(mem::take(&mut key), SaveFileValue::String(Rc::new(mem::take(&mut val))));
                        past_eq = false; // we reset the past_eq flag
                    }
                    else if !key.is_empty(){ // we have just a key { \n key \n }
                        stack.last_mut().unwrap().push(SaveFileValue::String(Rc::new(mem::take(&mut key))));
                    }
                }
                //TODO sometimes a text will precede an array like this color=rgb {} we should handle this
                ' ' | '\t' => { //syntax sugar we ignore, most of the time, unless...
                    if comment{
                        continue;
                    }
                    if !quotes{ // we are not in quotes, we check if we have a key value pair
                        // we are key=value <-here
                        if past_eq && !val.is_empty() { // in case {something=else something=else}
                            stack.last_mut().unwrap().insert(mem::take(&mut key), SaveFileValue::String(Rc::new(mem::take(&mut val))));
                            past_eq = false;
                        }
                        else if !key.is_empty() && !past_eq{ // in case { something something something } OR key =value we want to preserve the spaces
                            maybe_array = true;
                        }
                    }
                } 
                '=' => {
                    if comment{
                        continue;
                    }
                    maybe_array = false;
                    // if we have an assignment, we toggle adding from key to value
                    if quotes{
                        if past_eq{
                            val.push(c);
                        }else{
                            key.push(c);
                        }
                    }
                    else{
                        past_eq = true;
                    }
                }
                '#' => {
                    if quotes{
                        if past_eq{
                            val.push(c);
                        }else{
                            key.push(c);
                        }
                    }
                    else{
                        comment = true;
                    }
                }
                _ => { //the main content we append to the key or value
                    if comment{
                        continue;
                    }
                    if maybe_array { //we have a toggle that says that the last character was a space and key is not empty
                        stack.last_mut().unwrap().push(SaveFileValue::String(Rc::new(mem::take(&mut key))));
                        maybe_array = false;
                    }
                    if past_eq{
                        val.push(c);
                    }else{
                        key.push(c);
                    }
                }
            }
        }
        object = stack.pop().unwrap();
        object.rename(self.name.to_string());
        return object;
    }

    /// Skip the current section.
    /// Adds the length of the section to the offset and returns.
    /// This is useful if you are not interested in the contents of the section.
    /// 
    /// # Panics
    /// 
    /// Panics if the section is invalid.
    pub fn skip(&mut self){
        if !self.valid{
            panic!("Section {} is invalid", self.name);
        }
        self.invalidate();
        let mut depth = 0;
        let mut off = self.offset.get_internal_mut();
        for (ind, c) in self.contents[*off..].char_indices() {
            match c{
                '{' => {
                    depth += 1;
                }
                '}' => {
                    depth -= 1;
                    if depth == 0{
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
/// Some sections are skipped, some are parsed into [GameObject].
/// The choice is up to the user, after they are given the returned [Section] objects where they can check the section name using [Section::get_name].
/// 
/// # Example
/// 
/// ```
/// let save = SaveFile::new("save.ck3");
/// for section in save{
///    println!("Section: {}", section.get_name());
/// }
pub struct SaveFile{
    /// The contents of the save file, shared between all sections
    contents: Rc<String>,
    /// A single shared byte offset for all sections inside of [SaveFile::contents]
    offset: Shared<usize>,
}

impl SaveFile{

    /// Create a new SaveFile instance.
    /// The filename must be valid of course.
    pub fn open(filename: &str) -> SaveFile{
        let contents = std::fs::read_to_string(filename).unwrap();
        SaveFile{
            contents: Rc::new(contents),
            offset: Shared::wrap(0),
        }
    }

    pub fn open_compressed(filename: &str) -> SaveFile{
        let mut archive = ZipArchive::new(std::fs::File::open(filename).unwrap()).unwrap();
        let mut gamestate = archive.by_index(0).unwrap();
        if gamestate.is_dir(){
            panic!("Gamestate is a directory");
        }
        let mut contents = String::new();
        gamestate.read_to_string(&mut contents).unwrap();
        SaveFile{
            contents: Rc::new(contents),
            offset: Shared::wrap(0),
        }
    }

}

impl Iterator for SaveFile{

    type Item = Section;

    /// Get the next object in the save file
    /// If the file pointer has reached the end of the save file then it will return None.
    fn next(&mut self) -> Option<Section>{
        let mut key = String::new();
        let off = self.offset.get_internal_mut();
        for c in self.contents[*off..].chars(){
            match c{
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
        if key.is_empty(){
            return None;
        }
        return Some(Section::new(key, self.contents.clone(), self.offset.clone()));
    }
}

mod tests {

    #[allow(unused_imports)]
    use std::io::Write;

    #[allow(unused_imports)]
    use tempfile::NamedTempFile;

    #[allow(unused_imports)]
    use crate::game_object::GameObject;
    #[allow(unused_imports)]
    use crate::types::{Shared, Wrapper};

    #[test]
    fn test_save_file(){
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"
            test={
                test2={
                    test3=1
                }
            }
        ").unwrap();
        let mut save_file = super::SaveFile::open(file.path().to_str().unwrap());
        let object = save_file.next().unwrap().to_object();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.get_object_ref("test2");
        let test3 = test2.get_string_ref("test3");
        assert_eq!(*(test3) , "1".to_string());
    }

    #[test]
    fn test_save_file_array(){
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"
            test={
                test2={
                    1
                    2
                    3
                }
                test3={ 1 2 3}
            }
        ").unwrap();
        let mut save_file = super::SaveFile::open(file.path().to_str().unwrap());
        let object = save_file.next().unwrap().to_object();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.get_object_ref("test2");
        let test2_val = test2;
        assert_eq!(*(test2_val.get_index(0).unwrap().as_string()) , "1".to_string());
        assert_eq!(*(test2_val.get_index(1).unwrap().as_string()) , "2".to_string());
        assert_eq!(*(test2_val.get_index(2).unwrap().as_string()) , "3".to_string());
        let test3 = object.get_object_ref("test3");
        let test3_val = test3;
        assert_eq!(*(test3_val.get_index(0).unwrap().as_string()) , "1".to_string());
        assert_eq!(*(test3_val.get_index(1).unwrap().as_string()) , "2".to_string());
        assert_eq!(*(test3_val.get_index(2).unwrap().as_string()) , "3".to_string());
    }

    #[test]
    fn test_weird_syntax(){
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"
            test={
                test2={1=2
                    3=4}
                test3={1 2 
                    3}
                test4={1 2 3}
                test5=42
            }
        ").unwrap();
        let mut save_file = super::SaveFile::open(file.path().to_str().unwrap());
        let object = save_file.next().unwrap().to_object();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.get_object_ref("test2");
        assert_eq!(*(test2.get_string_ref("1")) , "2".to_string());
        assert_eq!(*(test2.get_string_ref("3")) , "4".to_string());
    }

    #[test]
    fn test_array_syntax(){
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"
            test={
                test2={ 1 2 3 }
            }
        ").unwrap();
        let mut save_file = super::SaveFile::open(file.path().to_str().unwrap());
        let object = save_file.next().unwrap().to_object();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.get_object_ref("test2");
        assert_eq!(*(test2.get_index(0).unwrap().as_string()) , "1".to_string());
        assert_eq!(*(test2.get_index(1).unwrap().as_string()) , "2".to_string());
        assert_eq!(*(test2.get_index(2).unwrap().as_string()) , "3".to_string());
        assert_eq!(test2.get_array_iter().len(), 3);
    }

    #[test]
    fn test_unnamed_obj(){
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"
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
        ").unwrap();
        let mut save_file = super::SaveFile::open(file.path().to_str().unwrap());
        let object = save_file.next().unwrap().to_object();
        let variables = object.get_object_ref("variables");
        let data = variables.get_object_ref("data");
        assert!(!data.is_empty());
        assert_ne!(data.get_array_iter().len(), 0)

    }

    #[test]
    fn test_example_1(){
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"
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
        }").unwrap();
        let mut save_file = super::SaveFile::open(file.path().to_str().unwrap());
        let object = save_file.next().unwrap().to_object();
        assert_eq!(object.get_name(), "3623".to_string());
        assert_eq!(*(object.get_string_ref("name")) , "dynn_Sao".to_string());
        let historical = object.get_object_ref("historical");
        assert_eq!(*(historical.get_index(0).unwrap().as_string()) , "4440".to_string());
        assert_eq!(*(historical.get_index(1).unwrap().as_string()) , "5398".to_string());
        assert_eq!(*(historical.get_index(2).unwrap().as_string()) , "6726".to_string());
        assert_eq!(*(historical.get_index(3).unwrap().as_string()) , "10021".to_string());
        assert_eq!(*(historical.get_index(4).unwrap().as_string()) , "33554966".to_string());
        assert_eq!(historical.get_array_iter().len(), 12);
    }

    #[test]
    fn test_space() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"
        test = {
            test2 = {
                test3 = 1
            }
            test4 = { a b c}
        }
        ").unwrap();
        let mut save_file = super::SaveFile::open(file.path().to_str().unwrap());
        let object = save_file.next().unwrap().to_object();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.get_object_ref("test2");
        let test3 = test2.get_string_ref("test3");
        assert_eq!(*(test3) , "1".to_string());
        let test4 = object.get_object_ref("test4");
        let test4_val = test4;
        assert_eq!(*(test4_val.get_index(0).unwrap().as_string()) , "a".to_string());
        assert_eq!(*(test4_val.get_index(1).unwrap().as_string()) , "b".to_string());
        assert_eq!(*(test4_val.get_index(2).unwrap().as_string()) , "c".to_string());
    }

    #[test]
    fn test_landed(){
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"
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
        ").unwrap();
        let mut save_file = super::SaveFile::open(file.path().to_str().unwrap());
        let object = save_file.next().unwrap().to_object();
        assert_eq!(object.get_name(), "c_derby".to_string());
        let b_derby = object.get_object_ref("b_derby");
        assert_eq!(*(b_derby.get_string_ref("province")) , "1621".to_string());
        let b_chesterfield = object.get_object_ref("b_chesterfield");
        assert_eq!(*(b_chesterfield.get_string_ref("province")) , "1622".to_string());
        let b_castleton = object.get_object_ref("b_castleton");
        assert_eq!(*(b_castleton.get_string_ref("province")) , "1623".to_string());
    }
}
