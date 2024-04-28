
use std::{cell::Ref, rc::Rc};

use crate::game_object::GameObject;

use crate::game_state::GameState;

use super::{Character, GameObjectDerived, Shared};

pub struct Player {
    pub name: Shared<String>,
    pub id: u32,
    pub character: Option<Shared<Character>> 
}

impl GameObjectDerived for Player {
    fn from_game_object(base: Ref<'_, GameObject>, game_state: &mut GameState) -> Self {
        let key = base.get("character").unwrap().as_string_ref().unwrap();
        Player {
            name: base.get("name").unwrap().as_string(),
            id: base.get("player").unwrap().as_string_ref().unwrap().parse::<u32>().unwrap(),
            character: Some(Rc::from(game_state.get_character(&key).clone()))
        }
    }

    fn dummy(id:u32) -> Self {
        Player {
            name: Rc::new("".to_owned().into()),
            id: id,
            character: None
        }
    }

    fn init(&mut self, base: Ref<'_, GameObject>, game_state: &mut GameState) {
        let key = base.get("character").unwrap().as_string_ref().unwrap();
        self.character = Some(Rc::from(game_state.get_character(&key).clone()));
        self.id = base.get("player").unwrap().as_string_ref().unwrap().parse::<u32>().unwrap();
    }

    fn get_id(&self) -> u32 {
        self.id
    }
}
