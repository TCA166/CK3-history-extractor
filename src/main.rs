use core::panic;
use dialoguer::{Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json;
use std::{
    env, fs,
    io::stdin,
    path::Path,
};

/// A submodule that provides opaque types commonly used in the project
mod types;

/// A submodule that provides the intermediate parsing interface for the save file.
/// The [save_file](crate::save_file) module uses [GameObject](crate::game_object::GameObject) to store the parsed data and structures in [structures](crate::structures) are initialized from these objects.
mod game_object;
use game_object::GameId;

/// A submodule that provides the macro save file parsing.
/// It provides objects for handling entire [save files](SaveFile) and [sections](Section) of save files.
mod save_file;
use save_file::SaveFile;

/// A submodule that provides the [GameState] object, which is used as a sort of a dictionary.
/// CK3 save files have a myriad of different objects that reference each other, and in order to allow for centralized storage and easy access, the [GameState] object is used.
mod game_state;
use game_state::GameState;

/// A submodule that provides [GameObjectDerived](crate::structures::GameObjectDerived) objects which are serialized and rendered into HTML.
/// You can think of them like frontend DB view objects into parsed save files.
mod structures;
use structures::{FromGameObject, Player};

/// The submodule responsible for creating the [minijinja::Environment] and loading of templates.
mod jinja_env;
use jinja_env::create_env;

/// A module for handling the display of the parsed data.
mod display;
use display::{
    Cullable, GameMap, Grapher, Localizer, Renderable, RenderableType, Renderer, Timeline,
};

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
/// 8. `--output` - A flag that tells the program where to output the rendered files.
/// 9. `--include` - A flag that tells the program where to find additional files to include in the rendering.
/// 10. `--no-interaction` - A flag that tells the program not to interact with the user.
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
    let mut no_interaction = false;
    // The output path, if provided by the user
    let mut output_path: Option<String> = None;
    // The game path and mod paths, if provided by the user
    let mut game_path: Option<String> = None;
    let mut include_paths: Vec<String> = Vec::new(); //the game path should be the first element
    let languages = vec![
        "english",
        "french",
        "german",
        "korean",
        "russian",
        "simp_chinese",
        "spanish",
    ];
    // The language to use for localization
    let mut language = "english".to_string();
    // The depth to render the player's history
    let mut depth = 3;
    // whether the game state should be dumped to json
    let mut dump = false;
    if args.len() < 2 {
        if atty::isnt(atty::Stream::Stdin) {
            //raw file contents
            stdin().read_line(&mut filename).unwrap();
            filename = filename.trim().to_string();
            if filename.is_empty() {
                panic!("No filename provided");
            }
        } else {
            //console interface only if we are in a terminal
            filename = Input::<String>::new()
                .with_prompt("Enter the save file path")
                .validate_with(|input: &String| -> Result<(), &str> {
                    let p = Path::new(input);
                    if p.exists() && p.is_file() {
                        Ok(())
                    } else {
                        Err("File does not exist")
                    }
                })
                .interact()
                .unwrap();
            compressed = Confirm::new()
                .with_prompt("Is the file compressed?")
                .default(false)
                .interact()
                .unwrap();
            game_path = Input::<String>::new()
                .with_prompt("Enter the game path [empty for None]")
                .allow_empty(true)
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.is_empty() {
                        return Ok(());
                    }
                    let p = Path::new(input);
                    if p.exists() && p.is_dir() {
                        Ok(())
                    } else {
                        Err("Path does not exist")
                    }
                })
                //.with_initial_text("/common/Crusader Kings III/game") //TODO this doesn't work for some reason, library issues?
                .interact()
                .map_or(None, |x| if x.is_empty() { None } else { Some(x) });
            depth = Input::<usize>::new()
                .with_prompt("Enter the rendering depth")
                .default(3)
                .interact()
                .unwrap();
            let language_selection = Select::new()
                .with_prompt("Choose the localization language")
                .items(&languages)
                .default(0)
                .interact()
                .unwrap();
            if language_selection != 0 {
                language = languages[language_selection].to_string();
            }
            let include_input = Input::<String>::new()
                .with_prompt("Enter the include paths separated by a coma [empty for None]")
                .allow_empty(true)
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.is_empty() {
                        return Ok(());
                    }
                    let paths: Vec<&str> = input.split(',').collect();
                    for p in paths.iter() {
                        let path = Path::new(p.trim());
                        if !path.exists() || !path.is_dir() {
                            return Err("Path does not exist");
                        }
                    }
                    Ok(())
                })
                .interact()
                .unwrap();
            if !include_input.is_empty() {
                include_paths = include_input
                    .split(',')
                    .map(|x| x.trim().to_string())
                    .collect();
            }
            output_path = Input::<String>::new()
                .with_prompt("Enter the output path [empty for cwd]")
                .allow_empty(true)
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.is_empty() {
                        return Ok(());
                    }
                    let p = Path::new(input);
                    if p.exists() && p.is_dir() {
                        Ok(())
                    } else {
                        Err("Path does not exist")
                    }
                })
                .interact()
                .map_or(None, |x| if x.is_empty() { None } else { Some(x) });
        }
    } else {
        //console interface
        filename = args[1].clone();
        // foreach argument above 1
        let mut iter = args.iter().skip(2);
        while let Some(arg) = iter.next() {
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
                    depth = depth_str
                        .parse::<usize>()
                        .expect("Depth argument must be a number");
                    println!("Setting depth to {}", depth)
                }
                "--game-path" => {
                    game_path = Some(
                        iter.next()
                            .expect("Game path argument requires a value")
                            .clone(),
                    );
                }
                "--zip" => {
                    compressed = true;
                }
                "--language" => {
                    //we don't validate the language here, args are trusted, if someone uses them to mess with the behaviour we let them
                    language = iter
                        .next()
                        .expect("Language argument requires a value")
                        .clone();
                    println!("Using language {}", language);
                }
                "--no-vis" => {
                    no_vis = true;
                }
                "--output" => {
                    output_path = Some(
                        iter.next()
                            .expect("Output path argument requires a value")
                            .clone(),
                    );
                    println!("Outputting to {}", output_path.as_ref().unwrap());
                }
                "--include" => {
                    let mut path = iter.next();
                    if path.is_none() {
                        panic!("Include argument requires a value");
                    }
                    while path.is_some() {
                        include_paths.push(path.unwrap().clone());
                        let next = iter.next();
                        if next.is_none() {
                            break;
                        }
                        path = next;
                    }
                }
                "--dump" => {
                    dump = true;
                }
                "--no-interaction" => {
                    no_interaction = true;
                }
                _ => {
                    println!("Unknown argument: {}", arg);
                }
            }
        }
    }
    let bar_style = ProgressStyle::default_bar().template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}").unwrap().progress_chars("#>-");
    //even though we don't need these for parsing, we load them here to error out early
    if game_path.is_some() {
        include_paths.insert(0, game_path.unwrap());
    }
    let mut localizer = Localizer::new();
    let mut map = None;
    if !include_paths.is_empty() {
        println!("Using game files from: {:?}", include_paths);
        let progress_bar = ProgressBar::new(include_paths.len() as u64);
        progress_bar.set_style(bar_style.clone());
        progress_bar.set_message(include_paths.last().unwrap().to_owned());
        for path in progress_bar.wrap_iter(include_paths.iter().rev()) {
            let loc_path = path.clone() + "/localization/" + language.as_str();
            localizer.add_from_path(loc_path);
            if !no_vis && map.is_none() {
                let map_data = path.clone() + "/map_data";
                let p = Path::new(&map_data);
                if p.exists() && p.is_dir() {
                    map = Some(GameMap::new(path));
                }
            }
        }
        progress_bar.finish_with_message("Game files loaded");
    }
    localizer.resolve();
    //initialize the save file
    println!("Ready for save parsing...");
    let save: SaveFile;
    if compressed {
        save = SaveFile::open_compressed(filename.as_str());
    } else {
        save = SaveFile::open(filename.as_str());
    }
    // this is sort of like the first round of filtering where we store the objects we care about
    let mut game_state: GameState = GameState::new();
    let mut players: Vec<Player> = Vec::new();
    let progress_bar = ProgressBar::new(save.len() as u64);
    progress_bar.set_style(bar_style);
    for mut i in progress_bar.wrap_iter(save.into_iter()){
        progress_bar.set_message(i.get_name().to_owned());
        match i.get_name() {
            //the order is kept consistent with the order in the save file
            "traits_lookup" => {
                let r = i.to_object();
                game_state.add_lookup(r.get_array_iter().map(|x| x.as_string()).collect());
            }
            "landed_titles" => {
                let r = i.to_object();
                let landed = r.get_object_ref("landed_titles");
                for v in landed.get_obj_iter() {
                    let o = v.1.as_object();
                    if o.is_none() {
                        // apparently this isn't a bug, its a feature. Thats how it is in the savefile v.0=none\n
                        continue;
                    }
                    game_state.add_title(o.unwrap());
                }
            }
            "dynasties" => {
                let r = i.to_object();
                for d in r.get_obj_iter() {
                    let o = d.1.as_object().unwrap();
                    if o.get_name() == "dynasty_house" || o.get_name() == "dynasties" {
                        for h in o.get_obj_iter() {
                            let house = h.1.as_object();
                            if house.is_none() {
                                continue;
                            }
                            game_state.add_dynasty(house.unwrap());
                        }
                    }
                }
            }
            "living" => {
                let r = i.to_object();
                for l in r.get_obj_iter() {
                    let chr = l.1.as_object();
                    if chr.is_some() {
                        game_state.add_character(chr.unwrap());
                    }
                }
            }
            "dead_unprunable" => {
                let r = i.to_object();
                for d in r.get_obj_iter() {
                    let chr = d.1.as_object();
                    if chr.is_some() {
                        game_state.add_character(chr.unwrap());
                    }
                }
            }
            "characters" => {
                let r = i.to_object();
                let dead_prunable = r.get("dead_prunable");
                if dead_prunable.is_some() {
                    for d in dead_prunable.unwrap().as_object().unwrap().get_obj_iter() {
                        let chr = d.1.as_object();
                        if chr.is_some() {
                            game_state.add_character(chr.unwrap());
                        }
                    }
                }
            }
            "vassal_contracts" => {
                let r = i.to_object();
                let active = r.get_object_ref("active");
                for contract in active.get_obj_iter() {
                    let val = contract.1.as_object();
                    if val.is_some() {
                        game_state.add_contract(
                            &contract.0.parse::<GameId>().unwrap(),
                            &val.unwrap().get("vassal").unwrap().as_id(),
                        )
                    }
                }
            }
            "religion" => {
                let r = i.to_object();
                let faiths = r.get_object_ref("faiths");
                for f in faiths.get_obj_iter() {
                    game_state.add_faith(f.1.as_object().unwrap());
                }
            }
            "culture_manager" => {
                let r = i.to_object();
                let cultures = r.get_object_ref("cultures");
                for c in cultures.get_obj_iter() {
                    game_state.add_culture(c.1.as_object().unwrap());
                }
            }
            "character_memory_manager" => {
                let r = i.to_object();
                let database = r.get_object_ref("database");
                for d in database.get_obj_iter() {
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
            "artifacts" => {
                let artifacts = i.to_object();
                let arr = artifacts.get_object_ref("artifacts");
                for a in arr.get_obj_iter() {
                    let a = a.1.as_object();
                    if a.is_none() {
                        continue;
                    }
                    game_state.add_artifact(a.unwrap());
                }
            }
            _ => {
                i.skip();
            }
        }
    }
    progress_bar.finish_with_message("Save parsing complete");
    //prepare things for rendering
    let grapher;
    if !no_vis {
        grapher = Some(Grapher::new(&game_state));
    } else {
        grapher = None;
    }
    let env = create_env(use_internal, map.is_some(), no_vis);
    let timeline;
    if !no_vis {
        let mut tm = Timeline::new(&game_state);
        tm.set_depth(depth, &localizer);
        timeline = Some(tm);
    } else {
        timeline = None;
    }
    for player in players.iter_mut() {
        //render each player
        println!("Processing {:?}", player.name);
        let mut folder_name = player.name.to_string() + "'s history";
        if output_path.is_some() {
            folder_name = output_path.as_ref().unwrap().clone() + "/" + folder_name.as_str();
        }
        create_dir_maybe(&folder_name);
        create_dir_maybe(format!("{}/characters", &folder_name).as_str());
        create_dir_maybe(format!("{}/dynasties", &folder_name).as_str());
        create_dir_maybe(format!("{}/titles", &folder_name).as_str());
        create_dir_maybe(format!("{}/faiths", &folder_name).as_str());
        create_dir_maybe(format!("{}/cultures", &folder_name).as_str());
        player.set_depth(depth, &localizer);
        println!("Tree traversed");
        let mut renderer = Renderer::new(&env, folder_name.clone(), &game_state, map.as_ref(), grapher.as_ref());
        let mut queue = vec![RenderableType::Player(player)];
        if !no_vis {
            timeline
                .as_ref()
                .unwrap()
                .render_all(&mut queue, &mut renderer);
        }
        while let Some(obj) = queue.pop() {
            obj.render_all(&mut queue, &mut renderer);
        }
        if atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout) && !no_interaction {
            open::that(format!("{}/index.html", folder_name)).unwrap();
        }
    }
    if dump {
        let json = serde_json::to_string_pretty(&game_state).unwrap();
        fs::write("game_state.json", json).unwrap();
    }
    if atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout) && !no_interaction {
        Input::<String>::new()
            .with_prompt("Press enter to exit")
            .allow_empty(true)
            .interact()
            .unwrap();
    }
}
