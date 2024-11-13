
use dialoguer::{Completion, Input, Select};
use clap_derive::Parser;

use std::{
    fs,
    path::Path
};

use super::steam::{get_ck3_path, SteamError, CK3_PATH};

const CK3_EXTENSION: &str = "ck3";

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

/// A [Completion] struct for save file names, that also acts as a list of save files in the current directory.
struct SaveFileNameCompletion {
    save_files: Vec<String>,
}

impl Default for SaveFileNameCompletion {
    fn default() -> Self {
        let mut res = Vec::new();
        let path = Path::new(".");
        if path.is_dir() {
            for entry in fs::read_dir(path).expect("Directory not found") {
                let entry = entry.expect("Unable to read entry").path();
                if entry.is_file() {
                    if let Some(ext) = entry.extension() {
                        if ext == CK3_EXTENSION {
                            res.push(entry.to_string_lossy().into_owned());
                        }
                    }
                }
            }
        }
        SaveFileNameCompletion { save_files: res }
    }
}

impl Completion for SaveFileNameCompletion {
    fn get(&self, input: &str) -> Option<String> {
        self.save_files.iter().find(|x| x.contains(input)).cloned()
    }
}

/// A function to validate the path input.
fn validate_path(input: &String) -> Result<(), &'static str> {
    if input.is_empty() {
        return Ok(());
    }
    let p = Path::new(input);
    if p.exists() {
        Ok(())
    } else {
        Err("Invalid path")
    }
}

/// A function to parse the language argument.
fn parse_lang_arg(input: &str) -> Result<&'static str, &'static str> {
    LANGUAGES.iter().find(|x| **x == input).map_or(Err("Invalid language"), |e| Ok(*e))
}

/// The arguments to the program.
#[derive(Parser)]
pub struct Args {
    /// The name of the save file to parse.
    pub filename: String,
    #[arg(short, long, default_value_t = 3, help="The depth to render the player's history.")]
    /// The depth to render the player's history.
    pub depth: usize,
    #[arg(short, long, default_value_t = LANGUAGES[0], value_parser = parse_lang_arg, help="The language to use for localization.")]
    /// The language to use for localization.
    pub language: &'static str,
    #[arg(short, long, default_value = None, help="The path to the game files.")]
    /// The path to the game files.
    pub game_path: Option<String>,
    #[arg(short, long, help="The paths to include in the rendering.")]
    /// The paths to include in the rendering.
    pub include: Vec<String>,
    #[arg(short, long, default_value = None, help="The output path for the rendered files.")]
    /// The output path for the rendered files.
    pub output: Option<String>,
    #[arg(long, default_value_t = false, help="A flag that tells the program to dump the game state to a json file.")]
    /// A flag that tells the program to dump the game state to a json file.
    pub dump: bool,
    #[arg(long, default_value_t = false, help="A flag that tells the program not to render any images.")]
    /// A flag that tells the program not to render any images.
    pub no_vis: bool,
    #[arg(short, long, default_value_t = false, help="A flag that tells the program not to interact with the user.")]
    /// A flag that tells the program not to interact with the user.
    pub no_interaction: bool,
    #[arg(short, long, default_value_t = false, help="A flag that tells the program to use the internal templates instead of the templates in the `templates` folder.")]
    /// A flag that tells the program to use the internal templates instead of the templates in the `templates` folder.
    pub use_internal: bool,
}

impl Args {
    /// Create the object based on user input.
    pub fn get_from_user() -> Self {
        //console interface only if we are in a terminal
        let completion = SaveFileNameCompletion::default();
        let filename = Input::<String>::new()
            .with_prompt("Enter the save file path")
            .validate_with(validate_path)
            .with_initial_text(completion.save_files.get(0).unwrap_or(&"".to_string()))
            .completion_with(&completion)
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
        let game_path = Input::<String>::new()
            .with_prompt("Enter the game path [empty for None]")
            .allow_empty(true)
            .validate_with(validate_path)
            .with_initial_text(ck3_path)
            .interact_text()
            .map_or(None, |x| if x.is_empty() { None } else { Some(x) });
        let depth = Input::<usize>::new()
            .with_prompt("Enter the rendering depth")
            .default(3)
            .interact()
            .unwrap();
        let include_input = Input::<String>::new()
            .with_prompt("Enter the include paths separated by a coma [empty for None]")
            .allow_empty(true)
            .validate_with(validate_path)
            .interact()
            .unwrap();
        let mut include_paths = Vec::new();
        if !include_input.is_empty() {
            include_paths = include_input
                .split(',')
                .map(|x| x.trim().to_string())
                .collect();
        }
        let mut language = LANGUAGES[0];
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
        let output_path = Input::<String>::new()
            .with_prompt("Enter the output path [empty for cwd]")
            .allow_empty(true)
            .validate_with(validate_path)
            .interact()
            .map_or(None, |x| if x.is_empty() { None } else { Some(x) });
        Args {
            filename,
            depth,
            language,
            game_path,
            include: include_paths,
            output: output_path,
            dump: false,
            no_vis: false,
            no_interaction: false,
            use_internal: false,
        }
    }
}
