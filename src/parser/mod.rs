/// A submodule that provides [jomini] abstractions
mod types;

/// A submodule that provides the intermediate parsing interface for the save file.
/// The parser uses [GameObject](crate::parser::game_object::GameObject) to store the parsed data and structures in [structures](crate::structures) are initialized from these objects.
mod game_object;
use std::fmt::{self, Debug, Display};

pub use game_object::{
    ConversionError, GameId, GameObjectArray, GameObjectMap, GameString, KeyError, SaveFileObject,
    SaveFileValue,
};

/// A submodule that provides the [SaveFile] object, which is used to store the entire save file.
mod save_file;
pub use save_file::{SaveFile, SaveFileError};

mod section;
pub use section::{Section, SectionError};

mod section_reader;
pub use section_reader::SectionReader;

/// A submodule that provides the [GameState] object, which is used as a sort of a dictionary.
/// CK3 save files have a myriad of different objects that reference each other, and in order to allow for centralized storage and easy access, the [GameState] object is used.
mod game_state;
pub use game_state::GameState;
use section_reader::SectionReaderError;

#[derive(Debug)]
pub enum ParsingError {
    SectionError(String),
    SaveFileError(SaveFileError),
    ConversionError(ConversionError),
    StructureError(KeyError),
    ReaderError(String),
}

impl<'a> From<SectionError<'a>> for ParsingError {
    fn from(e: SectionError<'a>) -> Self {
        ParsingError::SectionError(format!("{:?}", e))
    }
}

impl From<SaveFileError> for ParsingError {
    fn from(e: SaveFileError) -> Self {
        ParsingError::SaveFileError(e)
    }
}

impl From<ConversionError> for ParsingError {
    fn from(e: ConversionError) -> Self {
        ParsingError::ConversionError(e)
    }
}

impl From<KeyError> for ParsingError {
    fn from(value: KeyError) -> Self {
        ParsingError::StructureError(value)
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
            Self::ConversionError(err) => Display::fmt(err, f),
            Self::SaveFileError(err) => Display::fmt(err, f),
            Self::SectionError(err) => Display::fmt(err, f),
            Self::StructureError(err) => Display::fmt(err, f),
            Self::ReaderError(err) => Display::fmt(err, f),
        }
    }
}

/*
impl<'a> error::Error for ParsingError<'a> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::SectionError(err) => Some(err),
            Self::SaveFileError(err) => Some(err),
            Self::StructureError(err) => Some(err),
            Self::ConversionError(err) => Some(err),
        }
    }
}
*/

use super::{
    structures::{FromGameObject, Player},
    types::{HashMap, Wrapper, WrapperMut},
};

/// A function that processes a section of the save file.
/// Based on the given section, it will update the [GameState] object and the [Player] vector.
/// The [GameState] object is used to store all the data from the save file, while the [Player] vector is used to store the player data.
pub fn process_section(
    i: &mut Section,
    game_state: &mut GameState,
    players: &mut Vec<Player>,
) -> Result<(), ParsingError> {
    match i.get_name() {
        "meta_data" => {
            let parsed = i.parse()?;
            let map = parsed.as_map()?;
            game_state.set_current_date(
                map.get("meta_date")
                    .ok_or_else(|| KeyError::MissingKey("meta_date", map.clone()))?
                    .as_string()?,
            );
            game_state.set_offset_date(
                map.get("meta_real_date")
                    .ok_or_else(|| KeyError::MissingKey("meta_real_date", map.clone()))?
                    .as_string()?,
            );
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
            for (_, v) in map
                .get("landed_titles")
                .ok_or_else(|| KeyError::MissingKey("landed_titles", map.clone()))?
                .as_object()?
                .as_map()?
            {
                if let SaveFileValue::Object(o) = v {
                    game_state.add_title(o.as_map()?);
                }
            }
        }
        "county_manager" => {
            let parsed = i.parse()?;
            let map = parsed.as_map()?;
            // we create an association between the county key and the faith and culture of the county
            // this is so that we can easily add the faith and culture to the title, so O(n) instead of O(n^2)
            let mut key_assoc = HashMap::default();
            for (key, p) in map
                .get("counties")
                .ok_or_else(|| KeyError::MissingKey("counties", map.clone()))?
                .as_object()?
                .as_map()?
            {
                let p = p.as_object()?.as_map()?;
                let faith = game_state.get_faith(
                    &p.get("faith")
                        .ok_or_else(|| KeyError::MissingKey("faith", map.clone()))?
                        .as_id()?,
                );
                let culture = game_state.get_culture(
                    &p.get("culture")
                        .ok_or_else(|| KeyError::MissingKey("culture", map.clone()))?
                        .as_id()?,
                );
                key_assoc.insert(key.as_str(), (faith, culture));
            }
            for (_, title) in game_state.get_title_iter() {
                let key = title.get_internal().get_key();
                if key.is_none() {
                    continue;
                }
                let assoc = key_assoc.get(key.unwrap().as_str());
                if assoc.is_none() {
                    continue;
                }
                let (faith, culture) = assoc.unwrap();
                title
                    .get_internal_mut()
                    .add_county_data(culture.clone(), faith.clone())
            }
        }
        "dynasties" => {
            for (_, d) in i.parse()?.as_map()? {
                if let SaveFileObject::Map(o) = d.as_object()? {
                    if o.get_name() == "dynasty_house" || o.get_name() == "dynasties" {
                        for (_, h) in o {
                            match h {
                                SaveFileValue::Object(o) => {
                                    game_state.add_dynasty(o.as_map()?);
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
            for (_, l) in i.parse()?.as_map()? {
                match l {
                    SaveFileValue::Object(o) => {
                        game_state.add_character(o.as_map()?);
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        "dead_unprunable" => {
            for (_, d) in i.parse()?.as_map()? {
                if let SaveFileValue::Object(o) = d {
                    game_state.add_character(o.as_map()?);
                }
            }
        }
        "characters" => {
            if let Some(dead_prunable) = i.parse()?.as_map()?.get("dead_prunable") {
                for (_, d) in dead_prunable.as_object()?.as_map()? {
                    match d {
                        SaveFileValue::Object(o) => {
                            game_state.add_character(o.as_map()?);
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
                .ok_or_else(|| KeyError::MissingKey("database | active", map.clone()))?
                .as_object()?
                .as_map()?
            {
                if let SaveFileValue::Object(val) = contract {
                    let val = val.as_map()?;
                    game_state.add_contract(
                        &key.parse::<GameId>().unwrap(),
                        &val.get("vassal")
                            .ok_or_else(|| KeyError::MissingKey("val", val.clone()))?
                            .as_id()?,
                    )
                }
            }
        }
        "religion" => {
            let parsed = i.parse()?;
            let map = parsed.as_map()?;
            for (_, f) in map
                .get("faiths")
                .ok_or_else(|| KeyError::MissingKey("faiths", map.clone()))?
                .as_object()?
                .as_map()?
            {
                game_state.add_faith(f.as_object()?.as_map()?);
            }
        }
        "culture_manager" => {
            let parsed = i.parse()?;
            let map = parsed.as_map()?;
            let cultures = map
                .get("cultures")
                .ok_or_else(|| KeyError::MissingKey("cultures", map.clone()))?
                .as_object()?
                .as_map()?;
            for (_, c) in cultures {
                game_state.add_culture(c.as_object()?.as_map()?);
            }
        }
        "character_memory_manager" => {
            let parsed = i.parse()?;
            let map = parsed.as_map()?;
            for (_, d) in map
                .get("database")
                .ok_or_else(|| KeyError::MissingKey("database", map.clone()))?
                .as_object()?
                .as_map()?
            {
                if let SaveFileValue::Object(o) = d {
                    game_state.add_memory(o.as_map()?);
                }
            }
        }
        "played_character" => {
            let p = Player::from_game_object(i.parse()?.as_map()?, game_state);
            players.push(p);
        }
        "artifacts" => {
            let parsed = i.parse()?;
            let map = parsed.as_map()?;
            for (_, a) in map
                .get("artifacts")
                .ok_or_else(|| KeyError::MissingKey("artifacts", map.clone()))?
                .as_object()?
                .as_map()?
                .into_iter()
            {
                if let SaveFileValue::Object(o) = a {
                    game_state.add_artifact(o.as_map()?);
                }
            }
        }
        _ => {}
    }
    return Ok(());
}

#[cfg(test)]
mod tests {

    use jomini::{self, TextTape};

    use crate::parser::types::Tape;

    use super::*;

    fn get_test_obj(contents: &str) -> Result<Tape, jomini::Error> {
        Ok(Tape::Text(TextTape::from_slice(contents.as_bytes())?))
    }

    #[test]
    fn test_save_file() -> Result<(), Box<dyn std::error::Error>> {
        let tape = get_test_obj(
            "
            test={
                test2={
                    test3=1
                }
            }
        ",
        )?;
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_map()?;
        let test3 = test2.get("test3").unwrap().as_string()?;
        assert_eq!(*(test3), "1".to_string());
        return Ok(());
    }

    #[test]
    fn test_save_file_array() -> Result<(), Box<dyn std::error::Error>> {
        let tape = get_test_obj(
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
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object
            .as_map()
            .unwrap()
            .get("test2")
            .unwrap()
            .as_object()
            .unwrap();
        let test2_val = test2.as_array()?;
        assert_eq!(*(test2_val.get_index(0)?.as_string()?), "1".to_string());
        assert_eq!(*(test2_val.get_index(1)?.as_string()?), "2".to_string());
        assert_eq!(*(test2_val.get_index(2)?.as_string()?), "3".to_string());
        let test3 = object.as_map()?.get("test3").unwrap().as_object()?;
        let test3_val = test3.as_array()?;
        assert_eq!(*(test3_val.get_index(0)?.as_string()?), "1".to_string());
        assert_eq!(*(test3_val.get_index(1)?.as_string()?), "2".to_string());
        assert_eq!(*(test3_val.get_index(2)?.as_string()?), "3".to_string());
        Ok(())
    }

    #[test]
    fn test_weird_syntax() -> Result<(), Box<dyn std::error::Error>> {
        let tape = get_test_obj(
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
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(*(test2.get("1").unwrap().as_string()?), "2".to_string());
        assert_eq!(*(test2.get("3").unwrap().as_string()?), "4".to_string());
        Ok(())
    }

    #[test]
    fn test_array_syntax() -> Result<(), Box<dyn std::error::Error>> {
        let tape = get_test_obj(
            "
            test={
                test2={ 1 2 3 }
            }
        ",
        )
        .unwrap();
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_array()?;
        assert_eq!(*(test2.get_index(0)?.as_string()?), "1".to_string());
        assert_eq!(*(test2.get_index(1)?.as_string()?), "2".to_string());
        assert_eq!(*(test2.get_index(2)?.as_string()?), "3".to_string());
        assert_eq!(test2.len(), 3);
        Ok(())
    }

    #[test]
    fn test_unnamed_obj() -> Result<(), Box<dyn std::error::Error>> {
        let tape = get_test_obj(
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
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap().unwrap().parse().unwrap();
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
        let tape = get_test_obj("
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
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "3623".to_string());
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
        assert_eq!(*(historical.get_index(0)?.as_string()?), "4440".to_string());
        assert_eq!(*(historical.get_index(1)?.as_string()?), "5398".to_string());
        assert_eq!(*(historical.get_index(2)?.as_string()?), "6726".to_string());
        assert_eq!(
            *(historical.get_index(3)?.as_string()?),
            "10021".to_string()
        );
        assert_eq!(
            *(historical.get_index(4)?.as_string()?),
            "33554966".to_string()
        );
        assert_eq!(historical.len(), 12);
        Ok(())
    }

    #[test]
    fn test_space() -> Result<(), Box<dyn std::error::Error>> {
        let tape = get_test_obj(
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
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_map()?;
        let test3 = test2.get("test3").unwrap().as_string()?;
        assert_eq!(*(test3), "1".to_string());
        let test4 = object.as_map()?.get("test4").unwrap().as_object()?;
        let test4_val = test4.as_array()?;
        assert_eq!(*(test4_val.get_index(0)?.as_string()?), "a".to_string());
        assert_eq!(*(test4_val.get_index(1)?.as_string()?), "b".to_string());
        assert_eq!(*(test4_val.get_index(2)?.as_string()?), "c".to_string());
        Ok(())
    }

    #[test]
    fn test_landed() -> Result<(), Box<dyn std::error::Error>> {
        let tape = get_test_obj(
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
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "c_derby".to_string());
        let b_derby = object
            .as_map()?
            .get("b_derby")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(
            *(b_derby.get("province").unwrap().as_string()?),
            "1621".to_string()
        );
        let b_chesterfield = object
            .as_map()?
            .get("b_chesterfield")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(
            *(b_chesterfield.get("province").unwrap().as_string()?),
            "1622".to_string()
        );
        let b_castleton = object
            .as_map()?
            .get("b_castleton")
            .unwrap()
            .as_object()?
            .as_map()?;
        assert_eq!(
            *(b_castleton.get("province").unwrap().as_string()?),
            "1623".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_invalid_line() -> Result<(), Box<dyn std::error::Error>> {
        let tape = get_test_obj(
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
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object
            .as_map()?
            .get("test2")
            .unwrap()
            .as_object()?
            .as_map()?;
        let test3 = test2.get("test3").unwrap().as_string()?;
        assert_eq!(*(test3), "1".to_string());
        Ok(())
    }

    #[test]
    fn test_empty() {
        let tape = get_test_obj(
            "
            test={
            }
        ",
        )
        .unwrap();
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
    }

    #[test]
    fn test_arr_index() {
        let tape = get_test_obj(
            "
            duration={ 2 0=7548 1=2096 }
        ",
        )
        .unwrap();
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "duration".to_string());
        let arr = object.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_id().unwrap(), 7548);
    }

    #[test]
    fn test_multi_key() -> Result<(), Box<dyn std::error::Error>> {
        let tape = get_test_obj(
            "
        test={
            a=hello
            a=world
        }
        ",
        )
        .unwrap();
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap().unwrap().parse().unwrap();
        let arr = object.as_map()?.get("a").unwrap().as_object()?.as_array()?;
        assert_eq!(arr.len(), 2);
        Ok(())
    }

    #[test]
    fn test_invalid_syntax_1() {
        let tape = get_test_obj(
            "
        test={
            a=hello
            b
        }
        ",
        )
        .unwrap();
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap();
        assert!(object.unwrap().parse().is_err())
    }

    #[test]
    fn test_invalid_syntax_2() {
        let tape = get_test_obj(
            "
        test={
            b
            a=hello
        }
        ",
        )
        .unwrap();
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap();
        assert!(object.unwrap().parse().is_err())
    }
    #[test]
    fn test_invalid_syntax_3() {
        assert!(get_test_obj(
            "
        b
        ",
        )
        .is_err());
    }
    #[test]
    fn test_invalid_syntax_4() {
        assert!(get_test_obj(
            "
        b={
        ",
        )
        .is_err());
    }
}
