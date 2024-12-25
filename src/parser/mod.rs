/// A submodule that provides the intermediate parsing interface for the save file.
/// The parser uses [GameObject](crate::parser::game_object::GameObject) to store the parsed data and structures in [structures](crate::structures) are initialized from these objects.
mod game_object;
pub use game_object::{
    GameId, GameObjectArray, GameObjectMap, GameString, SaveFileObject, SaveFileValue,
};

/// A submodule that provides the [SaveFile] object, which is used to store the entire save file.
mod save_file;
pub use save_file::{SaveFile, SaveFileError};

/// A submodule that provides the [Section] object, which is used to store the parsed data of a section of the save file.
mod section;
use section::Section;

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
pub fn process_section(i: &mut Section, game_state: &mut GameState, players: &mut Vec<Player>) {
    match i.get_name() {
        "meta_data" => {
            let r = i.parse().unwrap();
            let r = r.as_map();
            game_state.set_current_date(r.get_string_ref("meta_date"));
            game_state.set_offset_date(r.get_string_ref("meta_real_date"));
        }
        //the order is kept consistent with the order in the save file
        "traits_lookup" => {
            let r = i.parse().unwrap();
            game_state.add_lookup(r.as_array().into_iter().map(|x| x.as_string()).collect());
        }
        "landed_titles" => {
            let r = i.parse().unwrap();
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
            let r = i.parse().unwrap();
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
            let r = i.parse().unwrap();
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
            let r = i.parse().unwrap();
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
            let r = i.parse().unwrap();
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
            let r = i.parse().unwrap();
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
            let r = i.parse().unwrap();
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
            let r = i.parse().unwrap();
            let faiths = r.as_map().get_object_ref("faiths").as_map();
            for (_, f) in faiths.into_iter() {
                game_state.add_faith(f.as_object().as_map());
            }
        }
        "culture_manager" => {
            let r = i.parse().unwrap();
            let cultures = r.as_map().get_object_ref("cultures").as_map();
            for (_, c) in cultures.into_iter() {
                game_state.add_culture(c.as_object().as_map());
            }
        }
        "character_memory_manager" => {
            let r = i.parse().unwrap();
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
            let r = i.parse().unwrap();
            let p = Player::from_game_object(r.as_map(), game_state);
            players.push(p);
        }
        "artifacts" => {
            let artifacts = i.parse().unwrap();
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
        _ => {
            i.skip();
        }
    }
}
