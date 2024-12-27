/// A submodule that provides [jomini] abstractions
mod types;

/// A submodule that provides the intermediate parsing interface for the save file.
/// The parser uses [GameObject](crate::parser::game_object::GameObject) to store the parsed data and structures in [structures](crate::structures) are initialized from these objects.
mod game_object;
pub use game_object::{
    GameId, GameObjectArray, GameObjectMap, GameString, SaveFileObject, SaveFileValue,
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
) -> Result<(), SectionError> {
    match i.get_name() {
        "meta_data" => {
            let r = i.parse()?;
            let r = r.as_map();
            game_state.set_current_date(r.get_string_ref("meta_date"));
            game_state.set_offset_date(r.get_string_ref("meta_real_date"));
        }
        //the order is kept consistent with the order in the save file
        "traits_lookup" => {
            let r = i.parse()?;
            game_state.add_lookup(r.as_array().into_iter().map(|x| x.as_string()).collect());
        }
        "landed_titles" => {
            let r = i.parse()?;
            let landed = r.as_map().get_object_ref("landed_titles").as_map();
            for (_, v) in landed.into_iter() {
                match v {
                    SaveFileValue::Object(o) => {
                        game_state.add_title(o.as_map());
                    }
                    _ => {
                        // apparently this isn't a bug, its a feature. Thats how it is in the savefile v.0=none\n
                        continue;
                    }
                }
            }
        }
        "county_manager" => {
            let r = i.parse()?;
            let counties = r.as_map().get_object_ref("counties").as_map();
            // we create an association between the county key and the faith and culture of the county
            // this is so that we can easily add the faith and culture to the title, so O(n) instead of O(n^2)
            let mut key_assoc = HashMap::default();
            for (key, p) in counties.into_iter() {
                let p = p.as_object().as_map();
                let faith = game_state.get_faith(&p.get("faith").unwrap().as_id());
                let culture = game_state.get_culture(&p.get("culture").unwrap().as_id());
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
            let r = i.parse()?;
            for (_, d) in r.as_map().into_iter() {
                match d.as_object() {
                    SaveFileObject::Map(o) => {
                        if o.get_name() == "dynasty_house" || o.get_name() == "dynasties" {
                            for (_, h) in o.into_iter() {
                                match h {
                                    SaveFileValue::Object(o) => {
                                        game_state.add_dynasty(o.as_map());
                                    }
                                    _ => {
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    SaveFileObject::Array(_) => {
                        continue;
                    }
                }
            }
        }
        "living" => {
            let r = i.parse()?;
            for (_, l) in r.as_map().into_iter() {
                match l {
                    SaveFileValue::Object(o) => {
                        let chr = o.as_map();
                        game_state.add_character(chr);
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        "dead_unprunable" => {
            let r = i.parse()?;
            for (_, d) in r.as_map().into_iter() {
                match d {
                    SaveFileValue::Object(o) => {
                        let chr = o.as_map();
                        game_state.add_character(chr);
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        "characters" => {
            let r = i.parse()?;
            let dead_prunable = r.as_map().get("dead_prunable");
            if dead_prunable.is_some() {
                for (_, d) in dead_prunable.unwrap().as_object().as_map().into_iter() {
                    match d {
                        SaveFileValue::Object(o) => {
                            let chr = o.as_map();
                            game_state.add_character(chr);
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
            let r = r.as_map();
            // if version <= 1.12 then the key is active, otherwise it is database, why paradox?
            let active = r
                .get("database")
                .or(r.get("active"))
                .unwrap()
                .as_object()
                .as_map();
            for (key, contract) in active.into_iter() {
                match contract {
                    SaveFileValue::Object(val) => game_state.add_contract(
                        &key.parse::<GameId>().unwrap(),
                        &val.as_map().get("vassal").unwrap().as_id(),
                    ),
                    _ => {
                        continue;
                    }
                }
            }
        }
        "religion" => {
            let r = i.parse()?;
            let faiths = r.as_map().get_object_ref("faiths").as_map();
            for (_, f) in faiths.into_iter() {
                game_state.add_faith(f.as_object().as_map());
            }
        }
        "culture_manager" => {
            let r = i.parse()?;
            let cultures = r.as_map().get_object_ref("cultures").as_map();
            for (_, c) in cultures.into_iter() {
                game_state.add_culture(c.as_object().as_map());
            }
        }
        "character_memory_manager" => {
            let r = i.parse()?;
            let database = r.as_map().get_object_ref("database").as_map();
            for (_, d) in database.into_iter() {
                match d {
                    SaveFileValue::Object(o) => {
                        game_state.add_memory(o.as_map());
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        "played_character" => {
            let r = i.parse()?;
            let p = Player::from_game_object(r.as_map(), game_state);
            players.push(p);
        }
        "artifacts" => {
            let artifacts = i.parse()?;
            let arr = artifacts.as_map().get_object_ref("artifacts").as_map();
            for (_, a) in arr.into_iter() {
                match a {
                    SaveFileValue::Object(o) => {
                        game_state.add_artifact(o.as_map());
                    }
                    _ => {
                        continue;
                    }
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
    fn test_save_file() {
        let tape = get_test_obj(
            "
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
        let test2 = object.as_map().get_object_ref("test2").as_map();
        let test3 = test2.get_string_ref("test3");
        assert_eq!(*(test3), "1".to_string());
    }

    #[test]
    fn test_save_file_array() {
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
        )
        .unwrap();
        let mut reader = SectionReader::new(&tape);
        let object = reader.next().unwrap().unwrap().parse().unwrap();
        assert_eq!(object.get_name(), "test".to_string());
        let test2 = object.as_map().get_object_ref("test2");
        let test2_val = test2.as_array();
        assert_eq!(
            *(test2_val.get_index(0).unwrap().as_string()),
            "1".to_string()
        );
        assert_eq!(
            *(test2_val.get_index(1).unwrap().as_string()),
            "2".to_string()
        );
        assert_eq!(
            *(test2_val.get_index(2).unwrap().as_string()),
            "3".to_string()
        );
        let test3 = object.as_map().get_object_ref("test3");
        let test3_val = test3.as_array();
        assert_eq!(
            *(test3_val.get_index(0).unwrap().as_string()),
            "1".to_string()
        );
        assert_eq!(
            *(test3_val.get_index(1).unwrap().as_string()),
            "2".to_string()
        );
        assert_eq!(
            *(test3_val.get_index(2).unwrap().as_string()),
            "3".to_string()
        );
    }

    #[test]
    fn test_weird_syntax() {
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
        let test2 = object.as_map().get_object_ref("test2").as_map();
        assert_eq!(*(test2.get_string_ref("1")), "2".to_string());
        assert_eq!(*(test2.get_string_ref("3")), "4".to_string());
    }

    #[test]
    fn test_array_syntax() {
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
        let test2 = object.as_map().get_object_ref("test2").as_array();
        assert_eq!(*(test2.get_index(0).unwrap().as_string()), "1".to_string());
        assert_eq!(*(test2.get_index(1).unwrap().as_string()), "2".to_string());
        assert_eq!(*(test2.get_index(2).unwrap().as_string()), "3".to_string());
        assert_eq!(test2.len(), 3);
    }

    #[test]
    fn test_unnamed_obj() {
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
        let variables = object.as_map().get_object_ref("variables").as_map();
        let data = variables.get_object_ref("data").as_array();
        assert_ne!(data.len(), 0)
    }

    #[test]
    fn test_example_1() {
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
            *(object.as_map().get_string_ref("name")),
            "dynn_Sao".to_string()
        );
        let historical = object.as_map().get_object_ref("historical").as_array();
        assert_eq!(
            *(historical.get_index(0).unwrap().as_string()),
            "4440".to_string()
        );
        assert_eq!(
            *(historical.get_index(1).unwrap().as_string()),
            "5398".to_string()
        );
        assert_eq!(
            *(historical.get_index(2).unwrap().as_string()),
            "6726".to_string()
        );
        assert_eq!(
            *(historical.get_index(3).unwrap().as_string()),
            "10021".to_string()
        );
        assert_eq!(
            *(historical.get_index(4).unwrap().as_string()),
            "33554966".to_string()
        );
        assert_eq!(historical.len(), 12);
    }

    #[test]
    fn test_space() {
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
        let test2 = object.as_map().get_object_ref("test2").as_map();
        let test3 = test2.get_string_ref("test3");
        assert_eq!(*(test3), "1".to_string());
        let test4 = object.as_map().get_object_ref("test4");
        let test4_val = test4.as_array();
        assert_eq!(
            *(test4_val.get_index(0).unwrap().as_string()),
            "a".to_string()
        );
        assert_eq!(
            *(test4_val.get_index(1).unwrap().as_string()),
            "b".to_string()
        );
        assert_eq!(
            *(test4_val.get_index(2).unwrap().as_string()),
            "c".to_string()
        );
    }

    #[test]
    fn test_landed() {
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
        let b_derby = object.as_map().get_object_ref("b_derby").as_map();
        assert_eq!(*(b_derby.get_string_ref("province")), "1621".to_string());
        let b_chesterfield = object.as_map().get_object_ref("b_chesterfield").as_map();
        assert_eq!(
            *(b_chesterfield.get_string_ref("province")),
            "1622".to_string()
        );
        let b_castleton = object.as_map().get_object_ref("b_castleton").as_map();
        assert_eq!(
            *(b_castleton.get_string_ref("province")),
            "1623".to_string()
        );
    }

    #[test]
    fn test_invalid_line() {
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
        let test2 = object.as_map().get_object_ref("test2").as_map();
        let test3 = test2.get_string_ref("test3");
        assert_eq!(*(test3), "1".to_string());
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
        assert_eq!(object.as_array().len(), 3);
        assert_eq!(object.as_array()[0].as_id(), 7548);
    }

    #[test]
    fn test_multi_key() {
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
        let arr = object.as_map().get_object_ref("a").as_array();
        assert_eq!(arr.len(), 2);
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
        let object = reader.next().unwrap().unwrap().parse();
        assert!(object.is_err())
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
        let object = reader.next().unwrap().unwrap().parse();
        assert!(object.is_err())
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
