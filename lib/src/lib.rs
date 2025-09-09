/// Core save file parser. Wrapper over the lower level [save_file::parser]
/// module and [save_file::structures].
///
/// ## Getting Started
///
/// The facade for the entire module is the [save_file::SaveFile] struct, which
/// returns a [save_file::SectionReader]. Through that reader you can easily
/// populate the [save_file::GameState] using the
/// [save_file::SectionReader::process_sections] method.
///
/// ## Iterating over sections
///
/// The parser was built, to allow you to choose which sections to parse.
/// While you can simply [process](save_file::SectionReader::process_sections),
/// a [reader](save_file::SectionReader), you can also iterate over the sections
/// using the [next](save_file::SectionReader::next) method. This way you can
/// make the choice whether to process the section or to skip it yourself.
///
/// ## Example
///
/// ```rust
/// use ck3_history_extractor_lib::save_file::SaveFile;
///
/// if let Ok(save_file) = SaveFile::open("/path/to/file/") {
///     let mut reader = save_file.section_reader(None).unwrap();
///     while let Some(section) = reader.next() {
///         // Process the section here
///     }
/// }
/// ```
pub mod save_file;

/// Serialization routines for game entity references
#[cfg(feature = "serde")]
pub mod derived_ref;

/// A submodule for handling the game data.
///
/// ## Getting Started
///
/// The entire module has a simple facade in [game_data::GameData]. It is
/// created via the [game_data::GameDataLoader] builder, which allows you to
/// 'add' paths to search through for game data. After you are done searching
/// for game files, you just [game_data::GameDataLoader::finalize] to get your
/// [game_data::GameData].
///
/// ## Example
///
/// ```rust
/// use ck3_history_extractor_lib::game_data::GameDataLoader;
///
/// let mut loader = GameDataLoader::new(false, "english");
/// loader.process_path("/path/to/game/data/");
/// loader.process_path("/path/to/some/mod");
/// let game_data = loader.finalize();
/// ```
pub mod game_data;

/// A module for handling the display of the parsed data.
///
/// ## Getting Started
///
/// The main feature of this module is the [display::Renderer], which uses the
/// [save_file::GameState], [game_data::GameData] and optionally
/// [display::Grapher] to render [display::Renderable] structs.
///
#[cfg(feature = "display")]
pub mod display;
