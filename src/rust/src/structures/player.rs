
use std::rc::Rc;

use crate::game_object::GameObject;

use crate::game_state::GameState;

use super::{Character, GameObjectDerived};

pub struct Player {
    pub name: Rc<String>,
    pub id: u32,
    pub character: Rc<Character> 
}

impl GameObjectDerived for Player {
    fn from_game_object(base: &GameObject, game_state: &GameState) -> Self {
        let key = base.get("character").unwrap().as_string().unwrap();
        Player {
            name: Rc::new(base.get("name").unwrap().as_string().unwrap().to_string()),
            id: base.get("player").unwrap().as_string().unwrap().parse::<u32>().unwrap(),
            character: Rc::from(*game_state.get_character(key).unwrap())
        }
    }

    fn type_name() -> &'static str {
        "player"
    }
}
