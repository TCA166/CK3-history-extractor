
use std::{cell::Ref, rc::Rc};

use crate::game_object::GameObject;

use crate::game_state::GameState;

use super::{Character, GameObjectDerived, Shared};

pub struct Player {
    pub name: Shared<String>,
    pub id: u32,
    pub character: Shared<Character> 
}

impl GameObjectDerived for Player {
    fn from_game_object(base: Ref<'_, GameObject>, game_state: &mut GameState) -> Self {
        let key = base.get("character").unwrap().as_string_ref().unwrap();
        Player {
            name: base.get("name").unwrap().as_string(),
            id: base.get("player").unwrap().as_string_ref().unwrap().parse::<u32>().unwrap(),
            character: Rc::from(game_state.get_character(&key).clone())
        }
    }

    fn type_name() -> &'static str {
        "player"
    }

    fn dummy() -> Self {
        Player {
            name: Rc::new("".to_owned().into()),
            id: 0,
            character: Rc::new(Character::dummy().into())
        }
    }

    fn init(&mut self, base: Ref<'_, GameObject>, game_state: &mut GameState) {
        let key = base.get("character").unwrap().as_string_ref().unwrap();
        self.character = Rc::from(game_state.get_character(&key).clone());
        self.id = base.get("player").unwrap().as_string_ref().unwrap().parse::<u32>().unwrap();
    }
}
