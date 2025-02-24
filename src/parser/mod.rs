/// A submodule that provides [jomini] abstractions.
/// This allows us to create an intermediate parsing interface with objects like
/// [SaveFile] and [Section] that hold these abstractions, however hiding if the
/// save file is binary or text. Thanks to this a user doesn't have to write
/// boilerplate code to handle both types of save files.
mod types;

use std::{
    error,
    fmt::{self, Debug, Display},
    num::ParseIntError,
};

/// A submodule that provides the parser output objects.
/// The parser uses [GameObject](crate::parser::game_object::SaveFileObject) to
/// store the parsed data and structures in [structures](crate::structures) are
/// initialized from these objects. This is our workaround for the lack of
/// reflection in Rust, and it puts one more layer of abstraction between the
/// parser and the structures. Jomini style would be to have the structures
/// directly initialized from the token tape, but that wouldn't play well with
/// the way we store everything in a central [GameState] object.
mod game_object;
use derive_more::From;
pub use game_object::{
    ConversionError, GameId, GameObjectCollection, GameObjectMap, GameObjectMapping, GameString,
    KeyError, SaveFileObject, SaveFileValue, SaveObjectError,
};

/// A submodule that provides the [SaveFile] object, which is used to store the
/// entire save file. This is essentially the front-end of the parser, handling
/// the IO and the such.
mod save_file;
use jomini::common::DateError;
pub use save_file::{SaveFile, SaveFileError};

/// A submodule that provides the [Section] object, which allows the user to
/// choose which sections should be parsed.
mod section;
pub use section::{Section, SectionError};

/// A submodule that provides the [yield_section] function, which is used to
/// iterate over the save file and return the next section.
mod section_reader;
pub use section_reader::yield_section;

/// A submodule that provides the [GameState] object, which is used as a sort of
/// a dictionary. CK3 save files have a myriad of different objects that
/// reference each other, and in order to allow for centralized storage and easy
/// access, the [GameState] object is used.
mod game_state;
pub use game_state::GameState;
use section_reader::SectionReaderError;

/// A submodule providing
mod tokens;

/// An error that occurred somewhere within the broadly defined parsing process.
#[derive(Debug, From)]
pub enum ParsingError {
    /// An error that occurred while parsing a section.
    SectionError(SectionError),
    /// An error that occurred while processing [SaveFileValue] objects.
    StructureError(SaveObjectError),
    /// An error that occurred while creating [Section] objects.
    ReaderError(String),
    /// An error that occurred during low level tape processing
    JominiError(jomini::Error),
    DateError(DateError),
}

impl From<ConversionError> for ParsingError {
    fn from(value: ConversionError) -> Self {
        ParsingError::StructureError(SaveObjectError::ConversionError(value))
    }
}

impl From<KeyError> for ParsingError {
    fn from(value: KeyError) -> Self {
        ParsingError::StructureError(SaveObjectError::KeyError(value))
    }
}

impl From<ParseIntError> for ParsingError {
    fn from(err: ParseIntError) -> Self {
        ParsingError::StructureError(SaveObjectError::ConversionError(err.into()))
    }
}

impl<'a, 'b: 'a> From<SectionReaderError<'b>> for ParsingError {
    fn from(value: SectionReaderError<'b>) -> Self {
        ParsingError::ReaderError(format!("{:?}", value))
    }
}

impl<'a> Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReaderError(err) => write!(f, "error during section reading: {}", err),
            val => write!(f, "{}", val),
        }
    }
}

impl<'a> error::Error for ParsingError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::SectionError(err) => Some(err),
            Self::StructureError(err) => Some(err),
            Self::JominiError(err) => Some(err),
            Self::DateError(err) => Some(err),
            _ => None,
        }
    }
}

use super::{
    structures::{FromGameObject, Player},
    types::HashMap,
};

/// A function that processes a section of the save file.
/// Based on the given section, it will update the [GameState] object and the
/// [Player] vector. The [GameState] object is used to store all the data from
/// the save file, while the [Player] vector is used to store the player data.
/// Essentially the fasade of the parser, that makes the choices on which
/// sections to parse and how to parse them.
pub fn process_section(
    i: &mut Section,
    game_state: &mut GameState,
    players: &mut Vec<Player>,
) -> Result<(), ParsingError> {
    match i.get_name() {
        "meta_data" => {
            let parsed = i.parse()?;
            let map = parsed.as_map()?;
            game_state
                .set_current_date(map.get_date("meta_date")?, map.get_date("meta_real_date")?);
        }
        //the order is kept consistent with the order in the save file
        "traits_lookup" => {
            let lookup: Result<Vec<_>, _> = i
                .parse()?
                .as_array()?
                .into_iter()
                .map(|x| x.as_string())
                .collect();
            game_state.add_lookup(lookup?);
        }
        "landed_titles" => {
            let parsed = i.parse()?;
            let map = parsed.as_map()?;
            for (key, v) in map.get_object("landed_titles")?.as_map()? {
                if let SaveFileValue::Object(o) = v {
                    game_state.add_title(&key.parse::<GameId>()?, o.as_map()?)?;
                }
            }
        }
        "county_manager" => {
            let parsed = i.parse()?;
            let map = parsed.as_map()?;
            // we create an association between the county key and the faith and culture of the county
            // this is so that we can easily add the faith and culture to the title, so O(n) instead of O(n^2)
            let mut key_assoc = HashMap::default();
            for (key, p) in map.get_object("counties")?.as_map()? {
                let p = p.as_object()?.as_map()?;
                let faith = game_state.get_faith(&p.get_game_id("faith")?);
                let culture = game_state.get_culture(&p.get_game_id("culture")?);
                key_assoc.insert(key.as_str(), (faith, culture));
            }
            game_state.add_county_data(key_assoc);
        }
        "dynasties" => {
            for (key, d) in i.parse()?.as_map()? {
                if let SaveFileObject::Map(o) = d.as_object()? {
                    if key == "dynasty_house" || key == "dynasties" {
                        for (dynasty_key, h) in o {
                            match h {
                                SaveFileValue::Object(o) => {
                                    game_state.add_dynasty(
                                        &dynasty_key.parse::<GameId>()?,
                                        o.as_map()?,
                                    )?;
                                }
                                _ => {
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }
        "living" => {
            for (key, l) in i.parse()?.as_map()? {
                match l {
                    SaveFileValue::Object(o) => {
                        game_state.add_character(&key.parse::<GameId>()?, o.as_map()?)?;
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        "dead_unprunable" => {
            for (key, d) in i.parse()?.as_map()? {
                if let SaveFileValue::Object(o) = d {
                    game_state.add_character(&key.parse::<GameId>()?, o.as_map()?)?;
                }
            }
        }
        "characters" => {
            if let Some(dead_prunable) = i.parse()?.as_map()?.get("dead_prunable") {
                for (key, d) in dead_prunable.as_object()?.as_map()? {
                    match d {
                        SaveFileValue::Object(o) => {
                            game_state.add_character(&key.parse::<GameId>()?, o.as_map()?)?;
                        }
                        _ => {
                            continue;
                        }
                    }
                }
            }
        }
        "vassal_contracts" => {
            let r = i.parse()?;
            let map = r.as_map()?;
            // if version <= 1.12 then the key is active, otherwise it is database, why paradox?
            for (key, contract) in map
                .get("database")
                .or(map.get("active"))
                .ok_or_else(|| KeyError::MissingKey("database".to_string(), map.clone()))?
                .as_object()?
                .as_map()?
            {
                if let SaveFileValue::Object(val) = contract {
                    let val = val.as_map()?;
                    game_state
                        .add_contract(&key.parse::<GameId>().unwrap(), &val.get_game_id("vassal")?)
                }
            }
        }
        "religion" => {
            let parsed = i.parse()?;
            let map = parsed.as_map()?;
            for (key, f) in map.get_object("faiths")?.as_map()? {
                game_state.add_faith(&key.parse::<GameId>()?, f.as_object()?.as_map()?)?;
            }
        }
        "culture_manager" => {
            let parsed = i.parse()?;
            let map = parsed.as_map()?;
            let cultures = map.get_object("cultures")?.as_map()?;
            for (key, c) in cultures {
                game_state.add_culture(&key.parse::<GameId>()?, c.as_object()?.as_map()?)?;
            }
        }
        "character_memory_manager" => {
            let parsed = i.parse()?;
            let map = parsed.as_map()?;
            for (key, d) in map.get_object("database")?.as_map()? {
                if let SaveFileValue::Object(o) = d {
                    game_state.add_memory(&key.parse::<GameId>()?, o.as_map()?)?;
                }
            }
        }
        "played_character" => {
            // TODO what about id?
            let p = Player::from_game_object(0, i.parse()?.as_map()?, game_state)?;
            players.push(p);
        }
        "artifacts" => {
            let parsed = i.parse()?;
            let map = parsed.as_map()?;
            for (key, a) in map.get_object("artifacts")?.as_map()?.into_iter() {
                if let SaveFileValue::Object(o) = a {
                    game_state.add_artifact(&key.parse::<GameId>()?, o.as_map()?)?;
                }
            }
        }
        _ => {
            i.skip()?;
        }
    }
    return Ok(());
}

#[cfg(test)]
mod tests {

    use jomini::{self, text::TokenReader};

    use super::{types::Tape, *};

    fn get_test_obj(contents: &str) -> Result<Tape, jomini::Error> {
        Ok(Tape::Text(TokenReader::from_slice(contents.as_bytes())))
    }

    #[test]
    fn test_save_file() -> Result<(), Box<dyn std::error::Error>> {
        let mut tape = get_test_obj(
            "
            test={
                test2={
                    test3=1
                }
            }
        ",
        )?;
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let object = section.parse().unwrap();
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(test2.get("test3").unwrap().as_integer()?, 1);
        return Ok(());
    }

    #[test]
    fn test_save_file_array() -> Result<(), Box<dyn std::error::Error>> {
        let mut tape = get_test_obj(
            "
            test={
                test2={
                    1
                    2
                    3
                }
                test3={ 1 2 3}
            }
        ",
        )?;
        let mut section = yield_section(&mut tape).unwrap()?;
        assert_eq!(section.get_name(), "test");
        let object = section.parse().unwrap();
        let test2 = object.as_map()?.get("test2").unwrap().as_object()?;
        let test2_val = test2.as_array()?;
        assert_eq!(test2_val.get_index(0)?.as_integer()?, 1);
        assert_eq!(test2_val.get_index(1)?.as_integer()?, 2);
        assert_eq!(test2_val.get_index(2)?.as_integer()?, 3);
        let test3 = object.as_map()?.get("test3").unwrap().as_object()?;
        let test3_val = test3.as_array()?;
        assert_eq!(test3_val.get_index(0)?.as_integer()?, 1);
        assert_eq!(test3_val.get_index(1)?.as_integer()?, 2);
        assert_eq!(test3_val.get_index(2)?.as_integer()?, 3);
        Ok(())
    }

    #[test]
    fn test_weird_syntax() -> Result<(), Box<dyn std::error::Error>> {
        let mut tape = get_test_obj(
            "
            test={
                test2={1=2
                    3=4}
                test3={1 2 
                    3}
                test4={1 2 3}
                test5=42
            }
        ",
        )
        .unwrap();
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let object = section.parse().unwrap();
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(test2.get("1").unwrap().as_integer()?, 2);
        assert_eq!(test2.get("3").unwrap().as_integer()?, 4);
        Ok(())
    }

    #[test]
    fn test_array_syntax() -> Result<(), Box<dyn std::error::Error>> {
        let mut tape = get_test_obj(
            "
            test={
                test2={ 1 2 3 }
            }
        ",
        )
        .unwrap();
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let object = section.parse().unwrap();
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_array()?;
        assert_eq!(test2.get_index(0)?.as_integer()?, 1);
        assert_eq!(test2.get_index(1)?.as_integer()?, 2);
        assert_eq!(test2.get_index(2)?.as_integer()?, 3);
        assert_eq!(test2.len(), 3);
        Ok(())
    }

    #[test]
    fn test_unnamed_obj() -> Result<(), Box<dyn std::error::Error>> {
        let mut tape = get_test_obj(
            "
        3623={
            name=\"dynn_Sao\"
            variables={
                data={ 
                        {
                            flag=\"ai_random_harm_cooldown\"
                            tick=7818
                            data={
                                type=boolean
                                identity=1
                            }
                        }
                        {
                            something_else=\"test\"
                        }
                    }
                }
            }
        }
        ",
        )
        .unwrap();
        let object = yield_section(&mut tape).unwrap().unwrap().parse().unwrap();
        let variables = object
            .as_map()?
            .get("variables")
            .unwrap()
            .as_object()?
            .as_map()?;
        let data = variables.get("data").unwrap().as_object()?.as_array()?;
        assert_ne!(data.len(), 0);
        Ok(())
    }

    #[test]
    fn test_example_1() -> Result<(), Box<dyn std::error::Error>> {
        let mut tape = get_test_obj("
        3623={
            name=\"dynn_Sao\"
            variables={
                data={ {
                        flag=\"ai_random_harm_cooldown\"
                        tick=7818
                        data={
                            type=boolean
                            identity=1
                        }
        
                    }
         }
            }
            found_date=750.1.1
            head_of_house=83939093
            dynasty=3623
            historical={ 4440 5398 6726 10021 33554966 50385988 77977 33583389 50381158 50425637 16880568 83939093 }
            motto={
                key=\"motto_with_x_I_seek_y\"
                variables={ {
                        key=\"1\"
                        value=\"motto_the_sword_word\"
                    }
         {
                        key=\"2\"
                        value=\"motto_bravery\"
                    }
         }
            }
            artifact_claims={ 83888519 }
        }").unwrap();
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "3623");
        let object = section.parse().unwrap();
        assert_eq!(
            *(object.as_map()?.get("name").unwrap().as_string()?),
            "dynn_Sao".to_string()
        );
        let historical = object
            .as_map()?
            .get("historical")
            .unwrap()
            .as_object()?
            .as_array()?;
        assert_eq!(historical.get_index(0)?.as_integer()?, 4440);
        assert_eq!(historical.get_index(1)?.as_integer()?, 5398);
        assert_eq!(historical.get_index(2)?.as_integer()?, 6726);
        assert_eq!(historical.get_index(3)?.as_integer()?, 10021);
        assert_eq!(historical.get_index(4)?.as_integer()?, 33554966);
        assert_eq!(historical.len(), 12);
        Ok(())
    }

    #[test]
    fn test_space() -> Result<(), Box<dyn std::error::Error>> {
        let mut tape = get_test_obj(
            "
        test = {
            test2 = {
                test3 = 1
            }
            test4 = { a b c}
        }
        ",
        )
        .unwrap();
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let object = section.parse().unwrap();
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(test2.get("test3").unwrap().as_integer()?, 1);
        let test4 = object.as_map()?.get("test4").unwrap().as_object()?;
        let test4_val = test4.as_array()?;
        assert_eq!(*(test4_val.get_index(0)?.as_string()?), "a".to_string());
        assert_eq!(*(test4_val.get_index(1)?.as_string()?), "b".to_string());
        assert_eq!(*(test4_val.get_index(2)?.as_string()?), "c".to_string());
        Ok(())
    }

    #[test]
    fn test_landed() -> Result<(), Box<dyn std::error::Error>> {
        let mut tape = get_test_obj(
            "
        c_derby = {
            color = { 255 50 20 }

            cultural_names = {
                name_list_norwegian = cn_djuraby
                name_list_danish = cn_djuraby
                name_list_swedish = cn_djuraby
                name_list_norse = cn_djuraby
            }

            b_derby = {
                province = 1621

                color = { 255 89 89 }

                cultural_names = {
                    name_list_norwegian = cn_djuraby
                    name_list_danish = cn_djuraby
                    name_list_swedish = cn_djuraby
                    name_list_norse = cn_djuraby
                }
            }
            b_chesterfield = {
                province = 1622

                color = { 255 50 20 }
            }
            b_castleton = {
                province = 1623

                color = { 255 50 20 }
            }
        }
        ",
        )
        .unwrap();
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "c_derby");
        let object = section.parse().unwrap();
        let b_derby = object
            .as_map()?
            .get("b_derby")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(b_derby.get("province").unwrap().as_integer()?, 1621);
        let b_chesterfield = object
            .as_map()?
            .get("b_chesterfield")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(b_chesterfield.get("province").unwrap().as_integer()?, 1622);
        let b_castleton = object
            .as_map()?
            .get("b_castleton")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(b_castleton.get("province").unwrap().as_integer()?, 1623);
        Ok(())
    }

    #[test]
    fn test_invalid_line() -> Result<(), Box<dyn std::error::Error>> {
        let mut tape = get_test_obj(
            "
            nonsense=idk
            test={
                test2={
                    test3=1
                }
            }
        ",
        )
        .unwrap();
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "test");
        let object = section.parse().unwrap();
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(test2.get("test3").unwrap().as_integer()?, 1);
        Ok(())
    }

    #[test]
    fn test_empty() {
        let mut tape = get_test_obj(
            "
            test={
            }
        ",
        )
        .unwrap();
        let object = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(object.get_name(), "test");
    }

    #[test]
    fn test_arr_index() {
        let mut tape = get_test_obj(
            "
            duration={ 2 0=7548 1=2096 }
        ",
        )
        .unwrap();
        let mut section = yield_section(&mut tape).unwrap().unwrap();
        assert_eq!(section.get_name(), "duration");
        let object = section.parse().unwrap();
        let arr = object.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_id().unwrap(), 7548);
    }

    #[test]
    fn test_multi_key() -> Result<(), Box<dyn std::error::Error>> {
        let mut tape = get_test_obj(
            "
        test={
            a=hello
            a=world
        }
        ",
        )
        .unwrap();
        let object = yield_section(&mut tape).unwrap().unwrap().parse().unwrap();
        let arr = object.as_map()?.get("a").unwrap().as_object()?.as_array()?;
        assert_eq!(arr.len(), 2);
        Ok(())
    }

    /*
    #[test]
    fn test_invalid_syntax_1() {
        let mut tape = get_test_obj(
            "
        test={
            a=hello
            b
        }
        ",
        )
        .unwrap();
        let object = yield_section(&mut tape).unwrap();
        assert!(object.unwrap().parse().is_ok())
    }

    #[test]
    fn test_invalid_syntax_2() {
        let mut tape = get_test_obj(
            "
        test={
            b
            a=hello
        }
        ",
        )
        .unwrap();
        let object = yield_section(&mut tape).unwrap();
        assert!(object.unwrap().parse().is_err())
    }
    */
}
