use std::{env, fs, time::SystemTime, io::{stdout, stdin, prelude::*}};

/// A submodule that provides opaque types commonly used in the project
mod types;

/// A submodule that provides the intermediate parsing interface for the save file.
/// The [crate::save_file] module uses [crate::game_object::GameObject] to store the parsed data and structures in [crate::structures] are initialized from these objects.
mod game_object;

/// A submodule that provides the macro save file parsing.
/// It provides objects for handling entire [save files](SaveFile) and [sections](Section) of save files.
mod save_file;
use save_file::SaveFile;

/// A submodule handling game localization.
mod localizer;
use localizer::Localizer;

/// A submodule that provides the [GameState] object, which is used as a sort of a dictionary.
/// CK3 save files have a myriad of different objects that reference each other, and in order to allow for centralized storage and easy access, the [GameState] object is used.
mod game_state;
use game_state::GameState;

/// A submodule that provides [GameObjectDerived] objects which are serialized and rendered into HTML.
/// You can think of them like frontend DB view objects into parsed save files.
mod structures;
use structures::{Player, Renderable, Renderer, Cullable};

/// A submodule that handles the creation of the minijinja [Environment] and loading of templates.
mod jinja_env;
use jinja_env::create_env;

use crate::{game_object::GameId, structures::FromGameObject};

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
/// 2. `--internal` - A flag that tells the program to use the internal templates instead of the templates in the `templates` folder.
/// 3. `--depth` - A flag that tells the program how deep to render the player's history. Defaults to 3.
/// 4. `--localization` - A flag that tells the program where to find the localization files. If not provided, the program will use a crude localization.
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
    if cfg!(debug_assertions) {
        env::set_var("RUST_BACKTRACE", "1");
    }
    // TODO add decompression for compressed save files
    //Get the staring time
    let start_time = SystemTime::now();
    //User IO
    let mut filename = String::new();
    let args: Vec<String> = env::args().collect();
    #[cfg(internal)]
    let mut use_internal = false;
    #[cfg(not(internal))]
    let use_internal = false;
    let mut localization_path = None;
    let mut depth = 3;
    if args.len() < 2{
        stdout().write_all(b"Enter the filename: ").unwrap();
        stdout().flush().unwrap();
        //raw file contents
        stdin().read_line(&mut filename).unwrap();
        filename = filename.trim().to_string();
    }
    else{
        filename = args[1].clone();
        // foreach argument above 1
        let mut iter = args.iter().skip(2);
        while let Some(arg) = iter.next(){
            match arg.as_str() {
                "--internal" => {
                    #[cfg(internal)]
                    {
                        println!("Using internal templates");
                        use_internal = true;
                    }
                    #[cfg(not(internal))]
                    panic!("Internal templates requested but not compiled in")
                }
                "--depth" => {
                    let depth_str = iter.next().expect("Depth argument requires a value");
                    depth = depth_str.parse::<usize>().expect("Depth argument must be a number");
                    println!("Setting depth to {}", depth)
                }
                "--localization" => {
                    localization_path = Some(iter.next().expect("Localization argument requires a value").clone());
                    println!("Using localization from {}", localization_path.as_ref().unwrap());
                } 
                _ => {
                    println!("Unknown argument: {}", arg);
                }
            
            }
        }
    }
    let localizer = Localizer::new(localization_path);
    //initialize the save file
    let save = SaveFile::new(filename.as_str()); // now we have an iterator we can work with that returns these large objects
    // this is sort of like the first round of filtering where we store the objects we care about
    let mut game_state:GameState = GameState::new();
    let mut last_name = String::new();
    let mut players:Vec<Player> = Vec::new();
    //MAYBE add multiprocessing? mutlithreading?
    for mut i in save.into_iter(){
        if i.get_name() != last_name{
            print!("{:?}\n", i.get_name());
            stdout().flush().unwrap();
            last_name = i.get_name().to_string().clone();
        }
        match i.get_name(){ //the order is kept consistent with the order in the save file
            "traits_lookup" => {
                let r = i.to_object();
                game_state.add_lookup(r.get_array_iter().map(|x| x.as_string()).collect());
            }
            "landed_titles" => {
                let r = i.to_object();
                let landed = r.get_object_ref("landed_titles");
                for v in landed.get_obj_iter(){
                    let o = v.1.as_object();
                    if o.is_none(){
                        // apparently this isn't a bug, its a feature. Thats how it is in the savefile v.0=none\n
                        continue;
                    }
                    game_state.add_title(o.unwrap());
                }
            }
            "dynasties" => {
                let r = i.to_object();
                for d in r.get_obj_iter(){
                    let o = d.1.as_object().unwrap();
                    if o.get_name() == "dynasty_house" || o.get_name() == "dynasties"{
                        for h in o.get_obj_iter(){
                            let house = h.1.as_object();
                            if house.is_none(){
                                continue;
                            }
                            game_state.add_dynasty(house.unwrap());
                        }
                    }
                }
            }
            "living" => {
                let r = i.to_object();
                for l in r.get_obj_iter(){
                    game_state.add_character(l.1.as_object().unwrap());
                }
            }
            "dead_unprunable" => {
                let r = i.to_object();
                for d in r.get_obj_iter(){
                    game_state.add_character(d.1.as_object().unwrap());   
                }
            }
            "vassal_contracts" => {
                let r = i.to_object();
                let active = r.get_object_ref("active");
                for contract in active.get_obj_iter(){
                    let val = contract.1.as_object();
                    if val.is_some(){
                        game_state.add_contract(&contract.0.parse::<GameId>().unwrap(), &val.unwrap().get("vassal").unwrap().as_id())
                    }
                }
            }
            "religion" => {
                let r = i.to_object();
                let faiths = r.get_object_ref("faiths");
                for f in faiths.get_obj_iter(){
                    game_state.add_faith(f.1.as_object().unwrap());
                }
            }
            "culture_manager" => {
                let r = i.to_object();
                let cultures = r.get_object_ref("cultures");
                for c in cultures.get_obj_iter(){
                    game_state.add_culture(c.1.as_object().unwrap());
                }
            }
            "character_memory_manager" => {
                let r = i.to_object();
                let database = r.get_object_ref("database");
                for d in database.get_obj_iter(){
                    let mem = d.1.as_object();
                    if mem.is_none() {
                        continue;
                    }
                    game_state.add_memory(mem.unwrap());
                }
            } 
            "played_character" => {
                let r = i.to_object();
                let p = Player::from_game_object(&r, &mut game_state);
                players.push(p);
            }
            _ => {
                i.skip();
            }
        }
    }
    println!("Savefile parsing complete");
    let env = create_env(use_internal);
    for player in players.iter_mut(){
        println!("Processing {:?}", player.name);
        let folder_name = player.name.to_string() + "'s history";
        create_dir_maybe(&folder_name);
        create_dir_maybe(format!("{}/characters", &folder_name).as_str());
        create_dir_maybe(format!("{}/dynasties", &folder_name).as_str());
        create_dir_maybe(format!("{}/titles", &folder_name).as_str());
        create_dir_maybe(format!("{}/faiths", &folder_name).as_str());
        create_dir_maybe(format!("{}/cultures", &folder_name).as_str());
        player.set_depth(depth, &localizer);
        let mut renderer = Renderer::new(&env, folder_name.clone());
        player.render_all(&mut renderer);
    }
    //Get the ending time
    let end_time = SystemTime::now();
    //Print the time taken
    println!("\nTime taken: {}s\n", end_time.duration_since(start_time).unwrap().as_secs());
}
