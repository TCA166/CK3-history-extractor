
use std::{collections::HashMap, rc::Rc};

use crate::game_object::GameObject;

use super::{Character, GameObjectDerived, GameObjectDerivedType};

pub struct Player {
    name: Rc<String>,
    id: u32,
    character: Rc<Character> 
}

impl GameObjectDerived for Player {
    fn from_game_object(base: &GameObject, game_state: &HashMap<String, HashMap<String, GameObjectDerivedType>>) -> Player {
        let characters = game_state.get("characters").unwrap();
        let key = base.get("character").unwrap().as_string().unwrap();
        Player {
            name: Rc::new(base.get("name").unwrap().as_string().unwrap().to_string()),
            id: base.get("player").unwrap().as_string().unwrap().parse::<u32>().unwrap(),
            character: characters.get(key).unwrap().get_as::<Character>().unwrap()
        }
    }
}
