use std::collections::HashMap;
use std::rc::Rc;
use serde::Serialize;
use serde::ser::SerializeStruct;
use super::{Character, GameObjectDerived};

pub struct Memory {
    pub date: Rc<String>,
    pub r#type: Rc<String>,
    pub participants: HashMap<Rc<String>, Rc<Character>>,
}

impl GameObjectDerived for Memory {
    fn from_game_object(base: &crate::game_object::GameObject, game_state: &crate::game_state::GameState) -> Self {
        let part = base.get("participants").unwrap().as_object().unwrap();
        let mut participants = HashMap::new();
        for k in part.get_keys(){
            let v = part.get(&k).unwrap();
            participants.insert(Rc::from(k), Rc::from(game_state.get_character(v.as_string().unwrap().as_str()).unwrap().clone()));
        }
        Memory{
            date: base.get("date").unwrap().as_string().unwrap(),
            r#type: base.get("type").unwrap().as_string().unwrap(),
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
