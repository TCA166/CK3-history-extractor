use clap::Parser;
use derive_more::From;
use human_panic::setup_panic;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde_json;
use std::{
    env, error,
    fmt::{self, Debug, Display, Formatter},
    fs,
    io::{stdin, stdout, IsTerminal},
    ops::Not,
    time::Duration,
};

use ck3_history_extractor_lib::{
    display::{GetPath, Renderer},
    game_data::{GameDataLoader, Localizable},
    parser::{process_section, yield_section, GameState, SaveFile, SaveFileError},
    structures::{GameObjectDerived, Player},
};

/// The submodule responsible for creating the [minijinja::Environment] and loading of templates.
mod jinja_env;
use jinja_env::create_env;

/// A submodule for handling the arguments passed to the program
mod args;
use args::Args;

/// A submodule for handling Steam integration
mod steam;

/// The interval at which the progress bars should update.
const INTERVAL: Duration = Duration::from_secs(1);

/// An error a user has caused. Shame on him.
#[derive(From, Debug)]
enum UserError {
    /// The program is not running in a terminal
    NoTerminal,
    /// The file does not exist
    FileDoesNotExist,
    /// An error occurred during file handling
    FileError(SaveFileError),
}

impl Display for UserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UserError::NoTerminal => write!(f, "The program is not running in a terminal"),
            UserError::FileDoesNotExist => write!(f, "The file does not exist"),
            UserError::FileError(e) => write!(f, "An error occurred during file handling: {}", e),
        }
    }
}

impl error::Error for UserError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            UserError::FileError(e) => Some(e),
            _ => None,
        }
    }
}

/// Main function. This is the entry point of the program.
///
/// # Process
///
/// 1. Reads user input through the command line arguments or prompts the user for input.
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
        include_paths.push(game_path);
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
    let mut game_state: GameState = GameState::default();
    let mut players: Vec<Player> = Vec::new();
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(spinner_style.clone());
    progress_bar.enable_steady_tick(INTERVAL);
    let mut tape = save.tape();
    while let Some(res) = yield_section(&mut tape) {
        let section = res.unwrap();
        progress_bar.set_message(section.get_name().to_owned());
        // if an error occured somewhere here, there's nothing we can do
        process_section(section, &mut game_state, &mut players).unwrap();
        progress_bar.inc(1);
    }
    progress_bar.finish_with_message("Save parsing complete");
    //prepare things for rendering
    game_state.localize(&mut data).unwrap();
    let grapher = args.no_vis.not().then(|| game_state.new_grapher());
    let timeline = args.no_vis.not().then(|| game_state.new_timeline());
    let mut env = create_env(
        args.use_internal,
        data.get_map().is_some(),
        args.no_vis,
        &data,
    );
    // a big progress bar to show the progress of rendering that contains multiple progress bars
    let rendering_progress_bar = MultiProgress::new();
    let player_progress = rendering_progress_bar.add(ProgressBar::new(players.len() as u64));
    player_progress.set_style(bar_style);
    player_progress.enable_steady_tick(INTERVAL);
    for player in player_progress.wrap_iter(players.iter_mut()) {
        player.localize(&mut data).unwrap();
        //render each player
        let folder_name = player.get_name().to_string() + "'s history";
        player_progress.set_message(format!("Rendering {}", folder_name));
        let path = args.output.join(folder_name);
        let mut renderer = Renderer::new(
            path.as_path(),
            &game_state,
            &data,
            grapher.as_ref(),
            args.depth,
        );
        let render_spinner = rendering_progress_bar.add(ProgressBar::new_spinner());
        render_spinner.set_style(spinner_style.clone());
        render_spinner.enable_steady_tick(INTERVAL);
        if !args.no_vis {
            render_spinner.inc(renderer.add_object(timeline.as_ref().unwrap()) as u64);
        }
        render_spinner.inc(renderer.add_object(player) as u64);
        renderer.render_all(&mut env);
        render_spinner.finish_with_message("Rendering complete");
        if stdin().is_terminal() && stdout().is_terminal() && !args.no_interaction {
            // no need to error out here, its just a convenience feature
            if let Err(e) = open::that(player.get_path(path.as_path())) {
                eprintln!("Error opening browser: {}", e);
            }
        }
        rendering_progress_bar.remove(&render_spinner);
    }
    player_progress.finish_with_message("Players rendered");
    if let Some(dump_path) = args.dump {
        let json = serde_json::to_string_pretty(&game_state).unwrap();
        fs::write(dump_path, json).unwrap();
    }
    if let Some(dump_path) = args.dump_data {
        let json = serde_json::to_string_pretty(&data).unwrap();
        fs::write(dump_path, json).unwrap();
    }
    return Ok(());
}
