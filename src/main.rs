use clap::Parser;
use human_panic::setup_panic;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde_json;
use std::{
    env,
    fmt::{self, Debug, Formatter},
    fs,
    io::{stdin, stdout, IsTerminal},
    time::Duration,
};

/// A submodule that provides opaque types commonly used in the project
mod types;

/// A submodule that handles save file parsing
mod parser;
use parser::{process_section, yield_section, GameState, SaveFile, SaveFileError};

/// A submodule that provides [GameObjectDerived](crate::structures::GameObjectDerived) objects which are serialized and rendered into HTML.
/// You can think of them like frontend DB view objects into parsed save files.
mod structures;
use structures::Player;

/// The submodule responsible for creating the [minijinja::Environment] and loading of templates.
mod jinja_env;
use jinja_env::create_env;

/// A module for handling the display of the parsed data.
mod display;
use display::{Cullable, Renderable, Renderer};

mod game_data;
use game_data::{GameDataLoader, Localizable};

/// A submodule for handling the arguments passed to the program
mod args;
use args::Args;

/// A submodule for handling Steam integration
mod steam;

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
    if !args.filename.exists() || !args.filename.is_file() {
        return Err(UserError::FileDoesNotExist);
    }
    let bar_style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-");
    let spinner_style = ProgressStyle::default_spinner()
        .template("[{elapsed_precise}] {spinner} {msg}")
        .unwrap()
        .tick_chars("|/-\\ ");
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
        for path in progress_bar.wrap_iter(include_paths.iter().rev()) {
            progress_bar.set_message(path.to_str().unwrap().to_owned());
            loader.process_path(path).unwrap();
        }
        progress_bar.finish_with_message("Game files loaded");
    }
    let mut data = loader.finalize();
    //initialize the save file
    let save = SaveFile::open(args.filename)?;
    // this is sort of like the first round of filtering where we store the objects we care about
    let mut game_state: GameState = GameState::new();
    let mut players: Vec<Player> = Vec::new();
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(spinner_style.clone());
    progress_bar.enable_steady_tick(INTERVAL);
    let mut tape = save.tape();
    while let Some(res) = yield_section(&mut tape) {
        let mut section = res.unwrap();
        progress_bar.set_message(section.get_name().to_owned());
        // if an error occured somewhere here, there's nothing we can do
        process_section(&mut section, &mut game_state, &mut players).unwrap();
        progress_bar.inc(1);
    }
    progress_bar.finish_with_message("Save parsing complete");
    //prepare things for rendering
    game_state.localize(&mut data);
    let grapher;
    if !args.no_vis {
        grapher = Some(game_state.new_grapher());
    } else {
        grapher = None;
    }
    let env = create_env(args.use_internal, data.get_map().is_some(), args.no_vis);
    let timeline;
    if !args.no_vis {
        let mut tm = game_state.new_timeline();
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
    for player in player_progress.wrap_iter(players.iter_mut()) {
        player.localize(&mut data);
        //render each player
        let folder_name = player.name.to_string() + "'s history";
        player_progress.set_message(format!("Rendering {}", folder_name));
        let path = args.output.join(folder_name);
        let cull_spinner = rendering_progress_bar.add(ProgressBar::new_spinner());
        cull_spinner.set_style(spinner_style.clone());
        cull_spinner.enable_steady_tick(INTERVAL);
        player.set_depth(args.depth);
        cull_spinner.finish_with_message("Tree traversed");
        let mut renderer =
            Renderer::new(&env, path.as_path(), &game_state, &data, grapher.as_ref());
        let render_spinner = rendering_progress_bar.add(ProgressBar::new_spinner());
        render_spinner.set_style(spinner_style.clone());
        render_spinner.enable_steady_tick(INTERVAL);
        if !args.no_vis {
            render_spinner.inc(renderer.render_all(timeline.as_ref().unwrap()));
        }
        render_spinner.inc(renderer.render_all(player));
        render_spinner.finish_with_message("Rendering complete");
        if stdin().is_terminal() && stdout().is_terminal() && !args.no_interaction {
            // no need to error out here, its just a convenience feature
            if let Err(e) = open::that(player.get_path(path.as_path())) {
                eprintln!("Error opening browser: {}", e);
            }
        }
        rendering_progress_bar.remove(&cull_spinner);
        rendering_progress_bar.remove(&render_spinner);
    }
    player_progress.finish_with_message("Players rendered");
    if let Some(dump_path) = args.dump {
        let json = serde_json::to_string_pretty(&game_state).unwrap();
        fs::write(dump_path, json).unwrap();
    }
    return Ok(());
}
