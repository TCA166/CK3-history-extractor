use std::{
    collections::HashMap,
    error,
    fmt::{self, Display},
    mem,
    path::Path,
};

use super::{
    super::parser::{
        yield_section, GameId, ParsingError, SaveFile, SaveFileError, SaveFileObject, SaveFileValue,
    },
    map::MapError,
    GameData, GameMap, Localizer,
};

/// An error that occurred while processing game data
#[derive(Debug)]
pub enum GameDataError {
    /// A file is missing at the provided path
    MissingFile(String),
    /// The data is invalid in some way with description
    InvalidData(&'static str),
    ParsingError(ParsingError),
    IOError(SaveFileError),
    MapError(MapError),
}

impl From<SaveFileError> for GameDataError {
    fn from(e: SaveFileError) -> Self {
        GameDataError::IOError(e)
    }
}

impl From<ParsingError> for GameDataError {
    fn from(e: ParsingError) -> Self {
        GameDataError::ParsingError(e)
    }
}

impl From<MapError> for GameDataError {
    fn from(e: MapError) -> Self {
        GameDataError::MapError(e)
    }
}

impl Display for GameDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameDataError::MissingFile(path) => {
                write!(f, "a file {} is missing", path)
            }
            GameDataError::InvalidData(reason) => Display::fmt(reason, f),
            GameDataError::ParsingError(e) => Display::fmt(e, f),
            GameDataError::IOError(e) => Display::fmt(e, f),
            GameDataError::MapError(e) => {
                write!(f, "error occurred while processing map data: {}", e)
            }
        }
    }
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

/// Creates a mapping from province ids to their barony title keys
fn create_title_province_map(file: &SaveFile) -> Result<HashMap<GameId, String>, ParsingError> {
    let mut tape = file.tape();
    let mut map = HashMap::default();
    while let Some(res) = yield_section(&mut tape) {
        let title_object = res?.parse()?;
        //DFS in the structure
        let mut stack = vec![title_object.as_map()?];
        while let Some(o) = stack.pop() {
            if let Some(pro) = o.get("province") {
                match pro {
                    // apparently pdx sometimes makes an oopsie and in the files the key is doubled, thus leading us to parse that as an array
                    SaveFileValue::Object(o) => {
                        map.insert(
                            o.as_array()?.get_index(0)?.as_id()?,
                            o.get_name()?.to_owned(),
                        );
                    }
                    s => {
                        map.insert(s.as_id()?, o.get_name()?.to_owned());
                    }
                }
            }
            for (_key, val) in o {
                match val {
                    SaveFileValue::Object(val) => match val {
                        SaveFileObject::Map(val) => {
                            stack.push(val);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }
    return Ok(map);
}

// File system stuff

const LOCALIZATION_SUFFIX: &str = "localization";

const MAP_PATH_SUFFIX: &str = "map_data";
const PROVINCES_SUFFIX: &str = "provinces.png";
const RIVERS_SUFFIX: &str = "rivers.png";
const DEFINITION_SUFFIX: &str = "definition.csv";

const PROVINCE_MAP_PATH: &str = "common/landed_titles/00_landed_titles.txt";

/// A loader for game data
pub struct GameDataLoader {
    no_vis: bool,
    language: &'static str,
    map: Option<GameMap>,
    localizer: Localizer,
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
            if map_path.exists() && map_path.is_dir() {
                if self.map.is_some() {
                    return Err(GameDataError::InvalidData("Multiple map directories found"));
                }
                let province_path = path.join(PROVINCE_MAP_PATH);
                if !province_path.is_file() {
                    return Err(GameDataError::MissingFile(
                        province_path.to_string_lossy().to_string(),
                    ));
                }
                let file = SaveFile::open(&province_path)?;
                let map = create_title_province_map(&file)?;
                self.map = Some(GameMap::new(
                    map_path.join(PROVINCES_SUFFIX),
                    map_path.join(RIVERS_SUFFIX),
                    map_path.join(DEFINITION_SUFFIX),
                    map,
                )?);
            }
        }
        Ok(())
    }

    /// Finalize the game data processing
    pub fn finalize(&mut self) -> GameData {
        self.localizer.resolve();
        GameData {
            map: self.map.take(),
            localizer: mem::take(&mut self.localizer),
        }
    }
}
