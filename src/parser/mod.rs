/// A submodule that provides the intermediate parsing interface for the save file.
/// The [save_file](crate::parser::save_file) module uses [GameObject](crate::parser::game_object::GameObject) to store the parsed data and structures in [structures](crate::structures) are initialized from these objects.
mod game_object;
pub use game_object::{GameId, GameObject, GameString, SaveFileValue};

/// A submodule that provides the raw save file parsing.
/// It provides objects for handling entire [save files](SaveFile) and [sections](Section) of save files.
mod save_file;
pub use save_file::SaveFile;
use save_file::Section;

/// A submodule that provides the [GameState] object, which is used as a sort of a dictionary.
/// CK3 save files have a myriad of different objects that reference each other, and in order to allow for centralized storage and easy access, the [GameState] object is used.
mod game_state;
pub use game_state::GameState;

use super::{
    structures::{FromGameObject, Player},
    types::{Wrapper, WrapperMut},
};
use std::collections::HashMap;

/// A function that processes a section of the save file.
/// Based on the given section, it will update the [GameState] object and the [Player] vector.
/// The [GameState] object is used to store all the data from the save file, while the [Player] vector is used to store the player data.
pub fn process_section(i: &mut Section, game_state: &mut GameState, players: &mut Vec<Player>) {
    match i.get_name() {
        "meta_data" => {
            let r = i.to_object();
            game_state.set_current_date(r.get("meta_date").unwrap().as_string());
            game_state.set_offset_date(r.get("meta_real_date").unwrap().as_string());
        }
        //the order is kept consistent with the order in the save file
        "traits_lookup" => {
            let r = i.to_object();
            game_state.add_lookup(r.get_array_iter().map(|x| x.as_string()).collect());
        }
        "landed_titles" => {
            let r = i.to_object();
            let landed = r.get_object_ref("landed_titles");
            for v in landed.get_obj_iter() {
                let o = v.1.as_object();
                if o.is_none() {
                    // apparently this isn't a bug, its a feature. Thats how it is in the savefile v.0=none\n
                    continue;
                }
                game_state.add_title(o.unwrap());
            }
        }
        "county_manager" => {
            let r = i.to_object();
            let counties = r.get_object_ref("counties");
            // we create an association between the county key and the faith and culture of the county
            // this is so that we can easily add the faith and culture to the title, so O(n) instead of O(n^2)
            let mut key_assoc = HashMap::new();
            for (key, p) in counties.get_obj_iter() {
                let p = p.as_object().unwrap();
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
            let r = i.to_object();
            for d in r.get_obj_iter() {
                let o = d.1.as_object().unwrap();
                if o.get_name() == "dynasty_house" || o.get_name() == "dynasties" {
                    for h in o.get_obj_iter() {
                        let house = h.1.as_object();
                        if house.is_none() {
                            continue;
                        }
                        game_state.add_dynasty(house.unwrap());
                    }
                }
            }
        }
        "living" => {
            let r = i.to_object();
            for l in r.get_obj_iter() {
                let chr = l.1.as_object();
                if chr.is_some() {
                    game_state.add_character(chr.unwrap());
                }
            }
        }
        "dead_unprunable" => {
            let r = i.to_object();
            for d in r.get_obj_iter() {
                let chr = d.1.as_object();
                if chr.is_some() {
                    game_state.add_character(chr.unwrap());
                }
            }
        }
        "characters" => {
            let r = i.to_object();
            let dead_prunable = r.get("dead_prunable");
            if dead_prunable.is_some() {
                for d in dead_prunable.unwrap().as_object().unwrap().get_obj_iter() {
                    let chr = d.1.as_object();
                    if chr.is_some() {
                        game_state.add_character(chr.unwrap());
                    }
                }
            }
        }
        "vassal_contracts" => {
            let r = i.to_object();
            // if version <= 1.12 then the key is active, otherwise it is database, why paradox?
            let active = r
                .get("database")
                .or(r.get("active"))
                .unwrap()
                .as_object()
                .unwrap();
            for contract in active.get_obj_iter() {
                let val = contract.1.as_object();
                if val.is_some() {
                    game_state.add_contract(
                        &contract.0.parse::<GameId>().unwrap(),
                        &val.unwrap().get("vassal").unwrap().as_id(),
                    )
                }
            }
        }
        "religion" => {
            let r = i.to_object();
            let faiths = r.get_object_ref("faiths");
            for f in faiths.get_obj_iter() {
                game_state.add_faith(f.1.as_object().unwrap());
            }
        }
        "culture_manager" => {
            let r = i.to_object();
            let cultures = r.get_object_ref("cultures");
            for c in cultures.get_obj_iter() {
                game_state.add_culture(c.1.as_object().unwrap());
            }
        }
        "character_memory_manager" => {
            let r = i.to_object();
            let database = r.get_object_ref("database");
            for d in database.get_obj_iter() {
                let mem = d.1.as_object();
                if mem.is_none() {
                    continue;
                }
                game_state.add_memory(mem.unwrap());
            }
        }
        "played_character" => {
            let r = i.to_object();
            let p = Player::from_game_object(&r, game_state);
            players.push(p);
        }
        "artifacts" => {
            let artifacts = i.to_object();
            let arr = artifacts.get_object_ref("artifacts");
            for a in arr.get_obj_iter() {
                let a = a.1.as_object();
                if a.is_none() {
                    continue;
                }
                game_state.add_artifact(a.unwrap());
            }
        }
        _ => {
            i.skip();
        }
    }
}
