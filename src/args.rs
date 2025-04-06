use clap_derive::Parser;
use derive_more::Display;
use dialoguer::{Completion, Input, MultiSelect, Select};

use std::{
    error,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use super::steam::{get_game_path, get_library_path, get_mod_paths, SteamError, CK3_PATH};

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

#[derive(Debug, Display)]
enum InvalidPath {
    #[display("invalid path (does not exist)")]
    InvalidPath,
    #[display("not a file")]
    NotAFile,
    #[display("not a directory")]
    NotADir,
}

impl error::Error for InvalidPath {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

/// A function to validate the file path input.
fn validate_file_path(input: &String) -> Result<(), InvalidPath> {
    if input.is_empty() {
        return Ok(());
    }
    let p = Path::new(input);
    if p.exists() {
        if p.is_file() {
            return Ok(());
        } else {
            return Err(InvalidPath::NotAFile);
        }
    } else {
        return Err(InvalidPath::InvalidPath);
    }
}

/// A function to validate the path input.
fn validate_dir_path(input: &String) -> Result<(), InvalidPath> {
    if input.is_empty() {
        return Ok(());
    }
    let p = Path::new(input);
    if p.exists() {
        if p.is_dir() {
            return Ok(());
        } else {
            return Err(InvalidPath::NotADir);
        }
    } else {
        return Err(InvalidPath::InvalidPath);
    }
}

/// A function to parse the language argument.
fn parse_lang_arg(input: &str) -> Result<&'static str, &'static str> {
    LANGUAGES
        .iter()
        .find(|x| **x == input)
        .map_or(Err("Invalid language"), |e| Ok(*e))
}

/// A function to parse the path argument.
fn parse_path_arg(input: &str) -> Result<PathBuf, &'static str> {
    let p = PathBuf::from(input);
    if p.exists() {
        Ok(p)
    } else {
        Err("Invalid path")
    }
}

/// The arguments to the program.
#[derive(Parser)]
pub struct Args {
    #[arg(value_parser = parse_path_arg)]
    /// The path to the save file.
    pub filename: PathBuf,
    #[arg(short, long, default_value_t = 3)]
    /// The depth to render the player's history.
    pub depth: usize,
    #[arg(short, long, default_value_t = LANGUAGES[0], value_parser = parse_lang_arg)]
    /// The language to use for localization.
    pub language: &'static str,
    #[arg(short, long, default_value = None, value_parser = parse_path_arg)]
    /// The path to the game files.
    pub game_path: Option<PathBuf>,
    #[arg(short, long, value_parser = parse_path_arg)]
    /// The paths to include in the rendering.
    pub include: Vec<PathBuf>,
    #[arg(short, long, default_value = ".", value_parser = parse_path_arg)]
    /// The output path for the rendered files.
    pub output: PathBuf,
    #[arg(long, default_value = None,)]
    /// A flag that tells the program to dump the game state to a json file.
    pub dump: Option<PathBuf>,
    #[arg(long,default_value = None,)]
    /// A path to a file to dump the game data to.
    pub dump_data: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    /// A flag that tells the program not to render any images.
    pub no_vis: bool,
    #[arg(short, long, default_value_t = false)]
    /// A flag that tells the program not to interact with the user.
    pub no_interaction: bool,
    #[arg(short, long, default_value_t = false)]
    /// A flag that tells the program to use the internal templates instead of the templates in the `templates` folder.
    pub use_internal: bool,
}

impl Args {
    /// Create the object based on user input.
    pub fn get_from_user() -> Self {
        println!("Welcome to CK3 save parser!\nTab autocompletes the query, arrows cycle through possible options, space toggles selection and enter confirms the selection.");
        //console interface only if we are in a terminal
        let completion = SaveFileNameCompletion::default();
        let filename = PathBuf::from(
            Input::<String>::new()
                .with_prompt("Enter the save file path")
                .validate_with(validate_file_path)
                .with_initial_text(completion.save_files.get(0).unwrap_or(&"".to_string()))
                .completion_with(&completion)
                .interact_text()
                .unwrap(),
        );
        let ck3_path;
        let mut mod_paths = Vec::new();
        match get_library_path() {
            Ok(p) => {
                ck3_path = get_game_path(&p).unwrap_or_else(|e| {
                    eprintln!("Error trying to find your CK3 installation: {}", e);
                    CK3_PATH.into()
                });
                get_mod_paths(&p, &mut mod_paths).unwrap_or_else(|e| {
                    eprintln!("Error trying to find your CK3 mods: {}", e);
                });
            }
            Err(e) => {
                ck3_path = CK3_PATH.into();
                if !matches!(e, SteamError::SteamDirNotFound | SteamError::CK3Missing) {
                    eprintln!("Error trying to find your CK3 installation: {}", e);
                }
            }
        };
        let game_path = Input::<String>::new()
            .with_prompt("Enter the game path [empty for None]")
            .allow_empty(true)
            .validate_with(validate_dir_path)
            .with_initial_text(ck3_path.to_string_lossy())
            .interact_text()
            .map_or(None, |x| {
                if x.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(x))
                }
            });
        let depth = Input::<usize>::new()
            .with_prompt("Enter the rendering depth")
            .default(3)
            .interact()
            .unwrap();
        let include_paths = if mod_paths.len() > 0 {
            let mod_selection = MultiSelect::new()
                .with_prompt("Select the mods to include")
                .items(&mod_paths)
                .interact()
                .unwrap();
            mod_selection
                .iter()
                .map(|i| mod_paths[*i].as_ref().clone())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
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
            .validate_with(validate_dir_path)
            .interact()
            .map(|x| {
                if x.is_empty() {
                    PathBuf::from(".")
                } else {
                    PathBuf::from(x)
                }
            })
            .unwrap();
        Args {
            filename,
            depth,
            language,
            game_path,
            include: include_paths,
            output: output_path,
            dump: None,
            dump_data: None,
            no_vis: false,
            no_interaction: false,
            use_internal: false,
        }
    }
}
