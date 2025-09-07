use std::{collections::HashMap, error, fs::read_dir, io, mem, path::Path};

use derive_more::{Display, From};

use super::{
    super::save_file::{
        parser::{
            types::{GameId, GameString},
            GameObjectCollection, ParsingError, SaveFileObject, SaveFileSection, SaveFileValue,
        },
        SaveFile, SaveFileError,
    },
    map::MapError,
    GameData, GameMap, Localizer,
};

/// An error that occurred while processing game data
#[derive(Debug, From, Display)]
pub enum GameDataError {
    /// A file is missing at the provided path
    #[display("a file {_0} is missing")]
    MissingFile(String),
    /// The data is invalid in some way with description
    #[display("the data is invalid: {_0}")]
    InvalidData(&'static str),
    ParsingError(ParsingError),
    IOError(SaveFileError),
    MapError(MapError),
}

impl error::Error for GameDataError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            GameDataError::IOError(e) => Some(e),
            GameDataError::ParsingError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for GameDataError {
    fn from(e: io::Error) -> Self {
        GameDataError::IOError(SaveFileError::from(e))
    }
}

/// Creates a mapping from province ids to their barony title keys
fn create_title_province_map(
    file: &SaveFile,
    out: &mut HashMap<GameId, GameString>,
) -> Result<(), ParsingError> {
    let mut tape = file.section_reader(None).unwrap();
    while let Some(res) = tape.next() {
        let section = res?;
        let name = GameString::from(section.get_name());
        //DFS in the structure
        let mut stack = if let SaveFileObject::Map(base) = section.parse()? {
            vec![(base, name)]
        } else {
            // if the base object is an array, something wonky is going on and we just politely retreat
            continue;
        };
        while let Some(entry) = stack.pop() {
            if let Some(pro) = entry.0.get("province") {
                match pro {
                    // apparently pdx sometimes makes an oopsie and in the files the key is doubled, thus leading us to parse that as an array
                    SaveFileValue::Object(o) => {
                        out.insert(o.as_array()?.get_index(0)?.as_id()?, entry.1);
                    }
                    s => {
                        out.insert(s.as_id()?, entry.1);
                    }
                }
            }
            for (key, val) in entry.0 {
                match val {
                    SaveFileValue::Object(val) => match val {
                        SaveFileObject::Map(val) => {
                            stack.push((val, key.into()));
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// File system stuff

const LOCALIZATION_SUFFIX: &str = "localization";

const MAP_PATH_SUFFIX: &str = "map_data";
const PROVINCES_SUFFIX: &str = "provinces.png";
const RIVERS_SUFFIX: &str = "rivers.png";
const DEFINITION_SUFFIX: &str = "definition.csv";

const PROVINCE_DIR_PATH: &str = "common/landed_titles/";

/// A loader for game data
pub struct GameDataLoader {
    no_vis: bool,
    language: &'static str,
    map: Option<GameMap>,
    localizer: Localizer,
    title_province_map: HashMap<GameId, GameString>,
}

impl GameDataLoader {
    /// Create a new game data loader with the given language and
    /// setting for whether to load visual data
    pub fn new(no_vis: bool, language: &'static str) -> Self {
        GameDataLoader {
            no_vis,
            language,
            map: None,
            localizer: Localizer::default(),
            title_province_map: HashMap::default(),
        }
    }

    /// Search the given path for localization and map data
    pub fn process_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), GameDataError> {
        let path = path.as_ref();
        let loc_path = path.join(LOCALIZATION_SUFFIX).join(self.language);
        if loc_path.exists() && loc_path.is_dir() {
            self.localizer.add_from_path(&loc_path);
        }
        if !self.no_vis {
            let map_path = path.join(MAP_PATH_SUFFIX);
            if !map_path.exists() || !map_path.is_dir() {
                return Ok(()); // this is not an error, just no map data
            }
            let province_dir_path = path.join(PROVINCE_DIR_PATH);
            if !province_dir_path.exists() || !province_dir_path.is_dir() {
                // I guess having a custom map with vanilla titles is fine, but not for us
                return Err(GameDataError::InvalidData(
                    "custom map without custom titles",
                ));
            }
            let dir = read_dir(&province_dir_path)?;
            for entry in dir {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    create_title_province_map(
                        &SaveFile::open(entry.path())?,
                        &mut self.title_province_map,
                    )?;
                }
            }
            self.map = Some(GameMap::new(
                map_path.join(PROVINCES_SUFFIX),
                map_path.join(RIVERS_SUFFIX),
                map_path.join(DEFINITION_SUFFIX),
                &self.title_province_map,
            )?);
        }
        Ok(())
    }

    /// Finalize the game data processing
    pub fn finalize(&mut self) -> GameData {
        self.localizer.remove_formatting();
        GameData {
            map: self.map.take(),
            localizer: mem::take(&mut self.localizer),
            title_province_map: mem::take(&mut self.title_province_map),
        }
    }
}
