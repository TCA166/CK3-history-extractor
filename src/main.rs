use std::cell::RefCell;
use std::time::SystemTime;
use std::io::prelude::*;
use std::io::{stdout, stdin};
use std::env;
use std::fs;

/// A submodule that provides the intermediate parsing interface for the save file.
/// The [super::save_file] module uses [GameObject] to store the parsed data and structures in [super::structures] are initialized from these objects.
mod game_object;

/// A submodule that provides the macro save file parsing.
/// It provides objects for handling entire [save files](SaveFile) and [Section]s of save files.
mod save_file;
use save_file::SaveFile;

use minijinja::Environment;

/// A submodule that provides the [GameState] object, which is used as a sort of a dictionary.
/// CK3 save files have a myriad of different objects that reference each other, and in order to allow for centralized storage and easy access, the [GameState] object is used.
mod game_state;
use game_state::GameState;

/// A submodule that provides [GameObjectDerived] objects which are serialized and rendered into HTML.
/// You can think of them like frontend DB view objects into parsed save files.
mod structures;
use structures::{Player, GameObjectDerived, Renderable};

/// A convenience function to create a directory if it doesn't exist, and do nothing if it does.
/// Also prints an error message if the directory creation fails.
fn create_dir_maybe(name: &str) {
    if let Err(err) = fs::create_dir_all(name) {
        if err.kind() != std::io::ErrorKind::AlreadyExists {
            println!("Failed to create folder: {}", err);
        }
    }
}

/// Main function. This is the entry point of the program.
/// 
/// # Arguments
/// 
/// 1. `filename` - The name of the save file to parse. If not provided as a command line argument, the program will prompt the user to enter it.
/// 
/// # Process
/// 
/// 1. Reads the save file name from user
/// 2. Parses the save file.
///     1. Initializes a [save_file::SaveFile] object using the provided file name
///     2. Iterates over the [save_file::Section] objects in the save file
///         If the section is of interest to us (e.g. `living`, `dead_unprunable`, etc.):
///         1. We parse the section into [game_object::GameObject]
///         2. We parse the [game_object::GameObject] into [structures::GameObjectDerived] objects
///         3. We store the objects in the [game_state::GameState] object
/// 3. Initializes a [minijinja::Environment] and loads the templates from the `templates` folder
/// 4. Foreach encountered [structures::Player] in game:
///     1. Creates a folder with the player's name
///     2. Renders the objects into HTML using the templates and writes them to the folder
/// 5. Prints the time taken to parse the save file
///
fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    //Get the staring time
    let start_time = SystemTime::now();
    //User IO
    let mut filename = String::new();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2{
        stdout().write_all(b"Enter the filename: ").unwrap();
        stdout().flush().unwrap();
        //raw file contents
        stdin().read_line(&mut filename).unwrap();
        filename = filename.trim().to_string();
    }
    else{
        filename = args[1].clone();
    }
    //initialize the save file
    let save = SaveFile::new(filename.as_str()); // now we have an iterator we can work with that returns these large objects
    // this is sort of like the first round of filtering where we store the objects we care about
    let mut game_state:GameState = GameState::new();
    let mut last_name = String::new();
    let mut players:Vec<Player> = Vec::new();
    for mut i in save{
        if i.get_name() != last_name{
            print!("{:?}\n", i.get_name());
            stdout().flush().unwrap();
            last_name = i.get_name().to_string().clone();
        }
        match i.get_name(){ //the order is kept consistent with the order in the save file
            "traits_lookup" => {
                game_state.add_lookup(i.to_object().unwrap().get_array_iter().map(|x| x.as_string()).collect());
            }
            "landed_titles" => {
                let o = i.to_object().unwrap();
                let landed = o.get_object_ref("landed_titles");
                for v in landed.get_obj_iter(){
                    let o = v.1.as_object_ref();
                    if o.is_none(){
                        // apparently this isn't a bug, its a feature. Thats how it is in the savefile v.0=none\n
                        continue;
                    }
                    game_state.add_title(o.unwrap());
                }
            }
            "dynasties" => {
                for d in i.to_object().unwrap().get_obj_iter(){
                    let o = d.1.as_object_ref().unwrap();
                    if o.get_name() == "dynasty_house" || o.get_name() == "dynasties"{
                        for h in o.get_obj_iter(){
                            let house = h.1.as_object_ref();
                            if house.is_none(){
                                continue;
                            }
                            game_state.add_dynasty(house.unwrap());
                        }
                    }
                }
            }
            "living" => {
                let o = i.to_object().unwrap();
                for l in o.get_obj_iter(){
                    game_state.add_character(l.1.as_object_ref().unwrap());
                }
            }
            "dead_unprunable" => {
                let o = i.to_object().unwrap();
                for d in o.get_obj_iter(){
                    game_state.add_character(d.1.as_object_ref().unwrap());
                }
            }
            "vassal_contracts" => {
                let o = i.to_object().unwrap();
                let active = o.get_object_ref("active");
                for contract in active.get_obj_iter(){
                    let val = contract.1.as_object_ref();
                    if val.is_some(){
                        game_state.add_contract(contract.0, val.unwrap().get_string_ref("vassal"))
                    }
                }
            }
            "religion" => {
                let o = i.to_object().unwrap();
                let faiths = o.get_object_ref("faiths");
                for f in faiths.get_obj_iter(){
                    game_state.add_faith(f.1.as_object_ref().unwrap());
                }
            }
            "culture_manager" => {
                let o = i.to_object().unwrap();
                let cultures = o.get_object_ref("cultures");
                for c in cultures.get_obj_iter(){
                    game_state.add_culture(c.1.as_object_ref().unwrap());
                }
            }
            "character_memory_manager" => {
                let o = i.to_object().unwrap();
                let database = o.get_object_ref("database");
                for d in database.get_obj_iter(){
                    let mem = d.1.as_object_ref();
                    if mem.is_none() {
                        continue;
                    }
                    game_state.add_memory(mem.unwrap());
                }
            } 
            "played_character" => {
                let p = Player::from_game_object(RefCell::new(i.to_object().unwrap()).borrow(), &mut game_state);
                players.push(p);
            }
            _ => {
                i.skip();
            }
        }
    }
    println!("Savefile parsing complete");
    let mut env = Environment::new();
    let h_template = fs::read_to_string("templates/homeTemplate.html").unwrap();
    env.add_template("homeTemplate.html", h_template.as_str()).unwrap();
    let c_template = fs::read_to_string("templates/charTemplate.html").unwrap();
    env.add_template("charTemplate.html", c_template.as_str()).unwrap();
    //TODO serialization is done multiple times, this is inefficient
    for player in players{
        println!("Processing {:?}", player.name.borrow());
        let folder_name = player.name.borrow().clone() + "'s history";
        create_dir_maybe(&folder_name);
        create_dir_maybe(format!("{}/characters", &folder_name).as_str());
        create_dir_maybe(format!("{}/dynasties", &folder_name).as_str());
        create_dir_maybe(format!("{}/titles", &folder_name).as_str());
        create_dir_maybe(format!("{}/faiths", &folder_name).as_str());
        create_dir_maybe(format!("{}/cultures", &folder_name).as_str());
        player.render_all(&env, &folder_name).unwrap();
    }
    //Get the ending time
    let end_time = SystemTime::now();
    //Print the time taken
    println!("\nTime taken: {}s\n", end_time.duration_since(start_time).unwrap().as_secs());
}
