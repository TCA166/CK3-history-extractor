use std::{env, fs, time::SystemTime, io::{stdout, stdin, prelude::*}};

/// A submodule that provides opaque types commonly used in the project
mod types;

/// A submodule that provides the intermediate parsing interface for the save file.
/// The [crate::save_file] module uses [crate::game_object::GameObject] to store the parsed data and structures in [crate::structures] are initialized from these objects.
mod game_object;
use game_object::GameId;

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

/// A submodule that provides [Renderable] and [Cullable] traits for objects that can be rendered.
mod renderer;
use renderer::{Renderer, Renderable, Cullable};

/// A submodule that provides [GameObjectDerived] objects which are serialized and rendered into HTML.
/// You can think of them like frontend DB view objects into parsed save files.
mod structures;
use structures::{Player, FromGameObject};

/// The submodule responsible for creating the minijinja [Environment] and loading of templates.
mod jinja_env;
use jinja_env::create_env;

/// Map handling submodule.
mod map;
use map::GameMap;

/// The graphing submodule that handles the creation of graphs from the game state.
mod graph;
use graph::Grapher;

use crate::timeline::Timeline;

mod timeline;


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
/// 4. `--game-path` - A flag that tells the program where to find the game files. If not provided, the program will use a crude localization.
/// 5. `--zip` - A flag that tells the program that the input file is compressed into an archive.
/// 6. `--language` - A flag that tells the program which language to use for localization. Defaults to `english`.
/// 7. `--no-vis` - A flag that tells the program not to render any images
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
    //Get the staring time
    let start_time = SystemTime::now();
    //User IO
    let mut filename = String::new();
    let args: Vec<String> = env::args().collect();
    // if we need to decompress the savefile
    let mut compressed = false;
    #[cfg(internal)]
    let mut use_internal = false;
    #[cfg(not(internal))]
    let use_internal = false;
    // if we don't want to render any images
    let mut no_vis = false;
    // The output path, if provided by the user
    let mut output_path:Option<String> = None;
    // The game path, if provided by the user
    let mut game_path = None; 
    // The language to use for localization
    let mut language = "english".to_string();
    // The depth to render the player's history
    let mut depth = 3;
    if args.len() < 2{
        print!("Enter the filename: ");
        stdout().flush().unwrap();
        //raw file contents
        stdin().read_line(&mut filename).unwrap();
        filename = filename.trim().to_string();
        print!("Enter the game path(You can just leave this empty): ");
        stdout().flush().unwrap();
        let mut inp = String::new();
        stdin().read_line(&mut inp).unwrap();
        inp = inp.trim().to_string();
        if inp.is_empty(){
            game_path = None;
        }
        else{
            game_path = Some(inp);
            println!("Using game files from {}", game_path.as_ref().unwrap());
        }
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
                "--game-path" => {
                    game_path = Some(iter.next().expect("Game path argument requires a value").clone());
                    println!("Using game files from {}", game_path.as_ref().unwrap());
                }
                "--zip" => {
                    compressed = true;
                }
                "--language" => {
                    language = iter.next().expect("Language argument requires a value").clone();
                    println!("Using language {}", language);
                }
                "--no-vis" => {
                    no_vis = true;
                }
                "--output" => {
                    output_path = Some(iter.next().expect("Output path argument requires a value").clone());
                    println!("Outputting to {}", output_path.as_ref().unwrap());
                }
                _ => {
                    println!("Unknown argument: {}", arg);
                }
            
            }
        }
    }
    let localization_path;
    let map;
    if game_path.is_some(){
        localization_path = Some(game_path.clone().unwrap() + "/localization/" + language.as_str());
        if !no_vis{
            map = Some(GameMap::new(&game_path.unwrap()));
        }
        else{
            map = None;
        }
    }
    else{
        localization_path = None;
        map = None;
    }
    let localizer = Localizer::new(localization_path);
    //initialize the save file
    let save:SaveFile;
    if compressed {
        save = SaveFile::open_compressed(filename.as_str());
    } else {
        save = SaveFile::open(filename.as_str());
    }
    // this is sort of like the first round of filtering where we store the objects we care about
    let mut game_state:GameState = GameState::new();
    let mut last_name = String::new();
    let mut players:Vec<Player> = Vec::new();
    println!("Ready for save parsing...");
    //MAYBE add multiprocessing? mutlithreading? Not necessary, not much IO happening
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
                    let chr = l.1.as_object();
                    if chr.is_some(){
                        game_state.add_character(chr.unwrap());
                    }
                }
            }
            "dead_unprunable" => {
                let r = i.to_object();
                for d in r.get_obj_iter(){
                    let chr = d.1.as_object();
                    if chr.is_some(){
                        game_state.add_character(chr.unwrap());  
                    }
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
    let grapher;
    if !no_vis{
        grapher = Some(Grapher::new(&game_state));
    }
    else{
        grapher = None;
    }
    let env = create_env(use_internal, map.is_some(), no_vis);
    let timeline;
    if !no_vis{
        let mut tm = Timeline::new(&game_state);
        tm.set_depth(depth, &localizer);
        timeline = Some(tm);
    } else{
        timeline = None;
    }
    for player in players.iter_mut(){
        println!("Processing {:?}", player.name);
        let mut folder_name = player.name.to_string() + "'s history";
        if output_path.is_some(){
            folder_name = output_path.as_ref().unwrap().clone() + "/" + folder_name.as_str();
        }
        create_dir_maybe(&folder_name);
        create_dir_maybe(format!("{}/characters", &folder_name).as_str());
        create_dir_maybe(format!("{}/dynasties", &folder_name).as_str());
        create_dir_maybe(format!("{}/titles", &folder_name).as_str());
        create_dir_maybe(format!("{}/faiths", &folder_name).as_str());
        create_dir_maybe(format!("{}/cultures", &folder_name).as_str());
        player.set_depth(depth, &localizer);
        let mut renderer = Renderer::new(&env, folder_name.clone());
        if !no_vis{
            timeline.as_ref().unwrap().render_all(&mut renderer, map.as_ref(), grapher.as_ref());
        }
        player.render_all(&mut renderer, map.as_ref(), grapher.as_ref());
    }
    //Get the ending time
    let end_time = SystemTime::now();
    //Print the time taken
    println!("\nTime taken: {}s\n", end_time.duration_since(start_time).unwrap().as_secs());
    print!("Press enter to exit...");
    stdout().flush().unwrap();
    let mut inp = String::new();
    stdin().read_line(&mut inp).unwrap();
}
