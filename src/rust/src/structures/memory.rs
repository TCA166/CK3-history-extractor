use std::cell::{RefCell, Ref};
use std::rc::Rc;
use serde::Serialize;
use serde::ser::SerializeStruct;
use super::{Character, GameObjectDerived, Shared};
use crate::game_object::GameObject;

pub struct Memory {
    pub date: Shared<String>,
    pub r#type: Shared<String>,
    pub participants: Vec<(Shared<String>, Shared<Character>)>,
}

impl GameObjectDerived for Memory {
    fn from_game_object(base: Ref<'_, GameObject>, game_state: &crate::game_state::GameState) -> Self {
        let part = base.get("participants").unwrap().as_object_ref().unwrap();
        let mut participants = Vec::new();
        for k in part.get_keys(){
            let v = part.get(&k).unwrap();
            participants.push((Rc::from(RefCell::from(k)), game_state.get_character(v.as_string_ref().unwrap().as_str()).unwrap().clone()));
        }
        Memory{
            date: base.get("date").unwrap().as_string(),
            r#type: base.get("type").unwrap().as_string(),
            participants: participants
        }
    }

    fn type_name() -> &'static str {
        "memory"
    }
}

impl Serialize for Memory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Memory", 3)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("type", &self.r#type)?;
        state.serialize_field("participants", &self.participants)?;
        state.end()
    }
}
