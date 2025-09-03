/// A submodule that handles save file parsing
pub mod parser;

pub mod derived_ref;

/// A submodule that provides objects which are serialized and rendered into HTML.
/// You can think of them like frontend DB view objects into parsed save files.
pub mod structures;

/// A submodule for handling the game data
pub mod game_data;

/// A module for handling the display of the parsed data.
pub mod display;

mod types;
