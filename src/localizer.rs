use std::{collections::HashMap, rc::Rc};
use std::fs;
use std::path::Path;
use yaml_rust::YamlLoader;

/// A function that demangles a generic name.
/// It will replace underscores with spaces and capitalize the first letter.
fn demangle_generic(input:&str) -> String{
    let mut s = input.replace("_", " ");
    let bytes = unsafe { s.as_bytes_mut() };
    bytes[0] = bytes[0].to_ascii_uppercase();
    s
}

pub struct Localizer{
    data: Option<HashMap<String, Rc<String>>>
}

impl Localizer{
    pub fn new(localization_src_path:Option<String>) -> Self{
        let mut hmap:Option<HashMap<String, Rc<String>>> = None;
        if localization_src_path.is_some() {
            let path = localization_src_path.unwrap();
            // get every file in the directory and subdirectories
            let mut data: HashMap<String, Rc<String>> = HashMap::new();
            let path = Path::new(&path);
            if path.is_dir() {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            //TODO check if the file has .yml extension
                            //TODO check if the file has localization data we care about
                            if let Ok(file_type) = entry.file_type() {
                                if file_type.is_file() {
                                    // read the file to string
                                    let contents = fs::read_to_string(entry.path()).unwrap();
                                    // parse the yaml
                                    println!("{:?}", contents);
                                    //FIXME are the yml pdx localization files actually yaml?
                                    let loc = YamlLoader::load_from_str(&contents).unwrap();
                                    let loc = loc.get(0).unwrap();
                                    for (key, value) in loc.as_hash().unwrap() {
                                        data.insert(key.as_str().unwrap().to_string(), Rc::new(value.as_str().unwrap().to_string()));
                                    }
                                }
                            }
                        }
                    }
                    hmap = Some(data);
                }
            }
        }
        println!("{:?}", hmap);
        Localizer{
            data: hmap
        }
    }

    pub fn localize(&self, key: &str) -> String{
        if self.data.is_none(){
            return demangle_generic(key)
        }
        let data = self.data.as_ref().unwrap();
        if data.contains_key(key){
            return data.get(key).unwrap().as_str().to_string()
        }
        demangle_generic(key)
    }
}
