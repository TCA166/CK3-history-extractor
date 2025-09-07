/// Lower level save parsing functionality.
/// Meant to be output structure agnostic, and focused on parsing into the
/// [intermediate representation](parser::SaveFileValue).
pub mod parser;

/// Output structures, like [structures::Character] and [structures::Title]
pub mod structures;

/// Module providing a parsed game state
mod game_state;
pub use game_state::GameState;

/// Lower level abstractions, [GameState] population
mod process_section;
pub use process_section::{Section, SectionReader};

/// Parser I/O and facade
mod save_file;
pub use save_file::{SaveFile, SaveFileError};
