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

/// A submodule for handling the game data
pub mod game_data;

/// A module for handling the display of the parsed data.
#[cfg(feature = "display")]
pub mod display;
