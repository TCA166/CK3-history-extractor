use dialoguer::{Input, Select};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde_json;
use human_panic::setup_panic;
use std::{
    env, fs,
    io::{stdin, stdout, IsTerminal},
    path::Path,
    time::Duration,
};

/// A submodule that provides opaque types commonly used in the project
mod types;

/// A submodule that handles save file parsing
mod parser;
use parser::{process_section, GameState, SaveFile};

/// A submodule that provides [GameObjectDerived](crate::structures::GameObjectDerived) objects which are serialized and rendered into HTML.
/// You can think of them like frontend DB view objects into parsed save files.
mod structures;
use structures::Player;

/// The submodule responsible for creating the [minijinja::Environment] and loading of templates.
mod jinja_env;
use jinja_env::create_env;

/// A module for handling the display of the parsed data.
mod display;
use display::{Cullable, GameMap, Grapher, Localizable, Localizer, Renderable, Renderer, Timeline};

/// A submodule for handling Steam integration
mod steam;
use steam::{get_ck3_path, SteamError, CK3_PATH};

/// The languages supported by the game.
const LANGUAGES: [&'static str; 7] = [
    "english",
    "french",
    "german",
    "korean",
    "russian",
    "simp_chinese",
    "spanish",
];

/// The name of the file to dump the game state to.
const DUMP_FILE: &str = "game_state.json";

/// The interval at which the progress bars should update.
const INTERVAL: Duration = Duration::from_secs(1);

const CK3_EXTENSION: &str = "ck3";

/// A helper function that lists all files with a certain extension in directory
fn find_files_with_extension(dir: &str, extension: &str) -> Vec<String> {
    let mut res = Vec::new();
    let path = Path::new(dir);
    if path.is_dir() {
        for entry in fs::read_dir(path).expect("Directory not found") {
            let entry = entry.expect("Unable to read entry").path();
            if entry.is_file() {
                if let Some(ext) = entry.extension() {
                    if ext == extension {
                        res.push(entry.to_string_lossy().into_owned());
                    }
                }
            }
        }
    }
    return res;
}

/// Main function. This is the entry point of the program.
///
/// # Arguments
///
/// 1. `filename` - The name of the save file to parse. If not provided as a command line argument, the program will prompt the user to enter it.
/// 2. `--internal` - A flag that tells the program to use the internal templates instead of the templates in the `templates` folder.
/// 3. `--depth` - A flag that tells the program how deep to render the player's history. Defaults to 3.
/// 4. `--game-path` - A flag that tells the program where to find the game files. If not provided, the program will use a crude localization.
/// 5. `--language` - A flag that tells the program which language to use for localization. Defaults to `english`.
/// 6. `--no-vis` - A flag that tells the program not to render any images
/// 7. `--output` - A flag that tells the program where to output the rendered files.
/// 8. `--include` - A flag that tells the program where to find additional files to include in the rendering.
/// 9. `--no-interaction` - A flag that tells the program not to interact with the user.
/// 10. `--dump` - A flag that tells the program to dump the game state to a json file.
///
/// # Process
///
/// 1. Reads the save file name from user
/// 2. Parses the save file.
///     1. Initializes a [SaveFile] object using the provided file name
///     2. Iterates over the Section objects in the save file
///         If the section is of interest to us (e.g. `living`, `dead_unprunable`, etc.):
///         1. We parse the section into [SaveFileObject](crate::parser::SaveFileObject) objects
///         2. We parse the objects into [Derived](structures::GameObjectDerived) objects
///         3. We store the objects in the [GameState] object
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
    setup_panic!();
    //User IO
    let mut filename = String::new();
    let args: Vec<String> = env::args().collect();
    #[cfg(feature = "internal")]
    let mut use_internal = false;
    #[cfg(not(feature = "internal"))]
    let use_internal = false;
    // if we don't want to render any images
    let mut no_vis = false;
    let mut no_interaction = false;
    // The output path, if provided by the user
    let mut output_path: Option<String> = None;
    // The game path and mod paths, if provided by the user
    let mut game_path: Option<String> = None;
    let mut include_paths: Vec<String> = Vec::new(); //the game path should be the first element
    let mut language = LANGUAGES[0]; // The language to use for localization
    let mut depth = 3; // The depth to render the player's history
                       // whether the game state should be dumped to json
    let mut dump = false;
    // user interface stuff
    if args.len() < 2 {
        if !stdin().is_terminal() {
            // simplified interface for non-terminal environments
            stdin().read_line(&mut filename).unwrap();
            filename = filename.trim().to_string();
            if filename.is_empty() {
                panic!("No filename provided");
            }
        } else {
            //console interface only if we are in a terminal
            let save_files = find_files_with_extension(".", CK3_EXTENSION);
            filename = Input::<String>::new()
                .with_prompt("Enter the save file path")
                .validate_with(|input: &String| -> Result<(), &str> {
                    let p = Path::new(input).canonicalize().unwrap();
                    if p.exists() && p.is_file() {
                        Ok(())
                    } else {
                        Err("File does not exist")
                    }
                })
                .with_initial_text(save_files.get(0).unwrap_or(&"".to_string()))
                .interact_text()
                .unwrap();
            let ck3_path = match get_ck3_path() {
                Ok(p) => p,
                Err(e) => {
                    match e {
                        SteamError::SteamDirNotFound => {
                            // we don't assume us being incompetent at finding the steam path is the user's fault
                            // so we don't print an error here
                            CK3_PATH.to_string()
                        }
                        SteamError::CK3Missing => {
                            // not having CK3 installed is also fine
                            "".to_string()
                        }
                        e => {
                            // but if we can't find the CK3 path for some other reason, we print an error
                            eprintln!("Error trying to find your CK3 installation: {:?}", e);
                            CK3_PATH.to_string()
                        }
                    }
                }
            };
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
                .with_initial_text(ck3_path)
                .interact_text()
                .map_or(None, |x| if x.is_empty() { None } else { Some(x) });
            depth = Input::<usize>::new()
                .with_prompt("Enter the rendering depth")
                .default(3)
                .interact()
                .unwrap();
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
            if game_path.is_some() || !include_paths.is_empty() {
                let language_selection = Select::new()
                    .with_prompt("Choose the localization language")
                    .items(&LANGUAGES)
                    .default(0)
                    .interact()
                    .unwrap();
                if language_selection != 0 {
                    language = LANGUAGES[language_selection];
                }
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
                    #[cfg(feature = "internal")]
                    {
                        println!("Using internal templates");
                        use_internal = true;
                    }
                    #[cfg(not(feature = "internal"))]
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
                "--language" => {
                    //we don't validate the language here, args are trusted, if someone uses them to mess with the behaviour we let them
                    language = iter.next().expect("Language argument requires a value");
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
                    while let Some(path) = iter.next() {
                        include_paths.push(path.clone());
                        let next = iter.next();
                        if next.is_none() {
                            break;
                        }
                    }
                }
                "--dump" => {
                    dump = true;
                }
                "--no-interaction" => {
                    no_interaction = true;
                }
                _ => {
                    eprintln!("Unknown argument: {}", arg);
                }
            }
        }
    }
    // arguments passed
    let p = Path::new(&filename);
    if !p.exists() || !p.is_file() {
        panic!("File does not exist");
    }
    let bar_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-");
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
        // "items" in this case are huge, 8s on my ssd, so we enable the steady tick
        progress_bar.enable_steady_tick(INTERVAL);
        progress_bar.set_message(include_paths.last().unwrap().to_owned());
        for path in progress_bar.wrap_iter(include_paths.iter().rev()) {
            let loc_path = path.clone() + "/localization/" + language;
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
    let save = SaveFile::open(filename.as_str()).unwrap();
    // this is sort of like the first round of filtering where we store the objects we care about
    let mut game_state: GameState = GameState::new();
    let mut players: Vec<Player> = Vec::new();
    let progress_bar = ProgressBar::new(save.len() as u64);
    progress_bar.set_style(bar_style.clone());
    for mut i in progress_bar.wrap_iter(save.into_iter()) {
        progress_bar.set_message(i.get_name().to_owned());
        process_section(&mut i, &mut game_state, &mut players);
    }
    progress_bar.finish_with_message("Save parsing complete");
    //prepare things for rendering
    game_state.localize(&localizer);
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
        tm.set_depth(depth);
        timeline = Some(tm);
    } else {
        timeline = None;
    }
    // a big progress bar to show the progress of rendering that contains multiple progress bars
    let rendering_progress_bar = MultiProgress::new();
    let player_progress = rendering_progress_bar.add(ProgressBar::new(players.len() as u64));
    player_progress.set_style(bar_style);
    player_progress.enable_steady_tick(INTERVAL);
    let spinner_style = ProgressStyle::default_spinner()
        .template("[{elapsed_precise}] {spinner} {msg}")
        .unwrap();
    for player in player_progress.wrap_iter(players.iter_mut()) {
        player.localize(&localizer);
        //render each player
        let mut folder_name = player.name.to_string() + "'s history";
        player_progress.set_message(format!("Rendering {}", folder_name));
        if let Some(output_path) = &output_path {
            folder_name = output_path.clone() + "/" + folder_name.as_str();
        }
        let cull_spinner = rendering_progress_bar.add(ProgressBar::new_spinner());
        cull_spinner.set_style(spinner_style.clone());
        cull_spinner.enable_steady_tick(INTERVAL);
        player.set_depth(depth);
        cull_spinner.finish_with_message("Tree traversed");
        let mut renderer = Renderer::new(
            &env,
            folder_name.clone(),
            &game_state,
            map.as_ref(),
            grapher.as_ref(),
        );
        let render_spinner = rendering_progress_bar.add(ProgressBar::new_spinner());
        render_spinner.set_style(spinner_style.clone());
        render_spinner.enable_steady_tick(INTERVAL);
        if !no_vis {
            render_spinner.inc(renderer.render_all(timeline.as_ref().unwrap()));
        }
        render_spinner.inc(renderer.render_all(player));
        render_spinner.finish_with_message("Rendering complete");
        if stdin().is_terminal() && stdout().is_terminal() && !no_interaction {
            open::that(player.get_path(&folder_name)).unwrap();
        }
        rendering_progress_bar.remove(&cull_spinner);
        rendering_progress_bar.remove(&render_spinner);
    }
    player_progress.finish_with_message("Players rendered");
    if dump {
        let json = serde_json::to_string_pretty(&game_state).unwrap();
        fs::write(DUMP_FILE, json).unwrap();
    }
    if stdin().is_terminal() && stdout().is_terminal() && !no_interaction {
        Input::<String>::new()
            .with_prompt("Press enter to exit")
            .allow_empty(true)
            .interact()
            .unwrap();
    }
}
