use clap::Parser;
use human_panic::setup_panic;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde_json;
use std::{
    env,
    fmt::{self, Debug, Formatter},
    fs,
    io::{stdin, stdout, IsTerminal},
    path::Path,
    time::Duration,
};

/// A submodule that provides opaque types commonly used in the project
mod types;

/// A submodule that handles save file parsing
mod parser;
use parser::{process_section, GameState, SaveFile, SaveFileError, SectionReader};

/// A submodule that provides [GameObjectDerived](crate::structures::GameObjectDerived) objects which are serialized and rendered into HTML.
/// You can think of them like frontend DB view objects into parsed save files.
mod structures;
use structures::Player;

/// The submodule responsible for creating the [minijinja::Environment] and loading of templates.
mod jinja_env;
use jinja_env::create_env;

/// A module for handling the display of the parsed data.
mod display;
use display::{Cullable, Grapher, Renderable, Renderer, Timeline};

mod game_data;
use game_data::{GameDataLoader, Localizable};

/// A submodule for handling the arguments passed to the program
mod args;
use args::Args;

/// A submodule for handling Steam integration
mod steam;

/// The name of the file to dump the game state to.
const DUMP_FILE: &str = "game_state.json";

/// The interval at which the progress bars should update.
const INTERVAL: Duration = Duration::from_secs(1);

enum UserError {
    NoTerminal,
    FileDoesNotExist,
    FileError(SaveFileError),
}

impl Debug for UserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UserError::NoTerminal => write!(f, "Not in a terminal"),
            UserError::FileDoesNotExist => write!(f, "File does not exist"),
            UserError::FileError(e) => {
                write!(f, "Error occurred during save file handling {:?}", e)
            }
        }
    }
}

impl From<SaveFileError> for UserError {
    fn from(value: SaveFileError) -> Self {
        return UserError::FileError(value);
    }
}

/// Main function. This is the entry point of the program.
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
fn main() -> Result<(), UserError> {
    if cfg!(debug_assertions) {
        env::set_var("RUST_BACKTRACE", "1");
    }
    setup_panic!();
    //User IO
    let args = if env::args().len() < 2 {
        if !stdout().is_terminal() {
            return Err(UserError::NoTerminal);
        }
        Args::get_from_user()
    } else {
        Args::parse()
    };
    // arguments passed
    let p = Path::new(&args.filename);
    if !p.exists() || !p.is_file() {
        return Err(UserError::FileDoesNotExist);
    }
    let bar_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-");
    let mut include_paths = args.include;
    //even though we don't need these for parsing, we load them here to error out early
    if let Some(game_path) = args.game_path {
        include_paths.insert(0, game_path);
    }
    let mut loader = GameDataLoader::new(args.no_vis, args.language);
    if !include_paths.is_empty() {
        println!("Using game files from: {:?}", include_paths);
        let progress_bar = ProgressBar::new(include_paths.len() as u64);
        progress_bar.set_style(bar_style.clone());
        // "items" in this case are huge, 8s on my ssd, so we enable the steady tick
        progress_bar.enable_steady_tick(INTERVAL);
        progress_bar.set_message(include_paths.last().unwrap().to_owned());
        for path in progress_bar.wrap_iter(include_paths.iter().rev()) {
            loader.process_path(path).unwrap();
        }
        progress_bar.finish_with_message("Game files loaded");
    }
    let mut data = loader.finalize();
    //initialize the save file
    let save = SaveFile::open(args.filename.as_str())?;
    let tape = save.tape().unwrap();
    let reader = SectionReader::new(&tape);
    // this is sort of like the first round of filtering where we store the objects we care about
    let mut game_state: GameState = GameState::new();
    let mut players: Vec<Player> = Vec::new();
    let progress_bar = ProgressBar::new(reader.len() as u64);
    progress_bar.set_style(bar_style.clone());
    for i in progress_bar.wrap_iter(reader.into_iter()) {
        let mut section = i.unwrap();
        progress_bar.set_message(section.get_name().to_owned());
        // if an error occured somewhere here, there's nothing we can do
        process_section(&mut section, &mut game_state, &mut players).unwrap();
    }
    progress_bar.finish_with_message("Save parsing complete");
    //prepare things for rendering
    game_state.localize(&mut data);
    let grapher;
    if !args.no_vis {
        grapher = Some(Grapher::new(&game_state));
    } else {
        grapher = None;
    }
    let env = create_env(args.use_internal, data.found_map(), args.no_vis);
    let timeline;
    if !args.no_vis {
        let mut tm = Timeline::new(&game_state);
        tm.set_depth(args.depth);
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
        .unwrap()
        .tick_chars("|/-\\ ");
    for player in player_progress.wrap_iter(players.iter_mut()) {
        player.localize(&mut data);
        //render each player
        let mut folder_name = player.name.to_string() + "'s history";
        player_progress.set_message(format!("Rendering {}", folder_name));
        if let Some(output_path) = &args.output {
            folder_name = output_path.clone() + "/" + folder_name.as_str();
        }
        let cull_spinner = rendering_progress_bar.add(ProgressBar::new_spinner());
        cull_spinner.set_style(spinner_style.clone());
        cull_spinner.enable_steady_tick(INTERVAL);
        player.set_depth(args.depth);
        cull_spinner.finish_with_message("Tree traversed");
        let mut renderer = Renderer::new(
            &env,
            folder_name.clone(),
            &game_state,
            data.get_map(),
            grapher.as_ref(),
        );
        let render_spinner = rendering_progress_bar.add(ProgressBar::new_spinner());
        render_spinner.set_style(spinner_style.clone());
        render_spinner.enable_steady_tick(INTERVAL);
        if !args.no_vis {
            render_spinner.inc(renderer.render_all(timeline.as_ref().unwrap()));
        }
        render_spinner.inc(renderer.render_all(player));
        render_spinner.finish_with_message("Rendering complete");
        if stdin().is_terminal() && stdout().is_terminal() && !args.no_interaction {
            open::that(player.get_path(&folder_name)).unwrap();
        }
        rendering_progress_bar.remove(&cull_spinner);
        rendering_progress_bar.remove(&render_spinner);
    }
    player_progress.finish_with_message("Players rendered");
    if args.dump {
        let json = serde_json::to_string_pretty(&game_state).unwrap();
        fs::write(DUMP_FILE, json).unwrap();
    }
    return Ok(());
}
