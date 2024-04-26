use std::cell::{RefCell, Ref};
use std::rc::Rc;
use serde::Serialize;
use serde::ser::SerializeStruct;
use super::{Character, GameObjectDerived, Shared};
use crate::game_object::GameObject;

pub struct Memory {
    pub id: u32,
    pub date: Shared<String>,
    pub r#type: Shared<String>,
    pub participants: Vec<(Shared<String>, Shared<Character>)>,
}

impl GameObjectDerived for Memory {
    fn from_game_object(base: Ref<'_, GameObject>, game_state: &mut crate::game_state::GameState) -> Self {
        let part = base.get("participants").unwrap().as_object_ref().unwrap();
        let mut participants = Vec::new();
        for k in part.get_keys(){
            let v = part.get(&k).unwrap();
            participants.push((Rc::from(RefCell::from(k)), game_state.get_character(v.as_string_ref().unwrap().as_str()).clone()));
        }
        Memory{
            date: base.get("date").unwrap().as_string(),
            r#type: base.get("type").unwrap().as_string(),
            participants: participants,
            id: base.get_name().parse::<u32>().unwrap()
        }
    }

    fn type_name() -> &'static str {
        "memory"
    }

    fn dummy() -> Self {
        Memory{
            date: Rc::new(RefCell::new("".to_owned())),
            r#type: Rc::new(RefCell::new("".to_owned())),
            participants: Vec::new(),
            id: 0
        }
    }

    fn init(&mut self, base: Ref<'_, GameObject>, game_state: &mut crate::game_state::GameState) {
        let part = base.get("participants").unwrap().as_object_ref().unwrap();
        let mut participants = Vec::new();
        for k in part.get_keys(){
            let v = part.get(&k).unwrap();
            participants.push((Rc::from(RefCell::from(k)), game_state.get_character(v.as_string_ref().unwrap().as_str()).clone()));
        }
        self.date = base.get("date").unwrap().as_string();
        self.r#type = base.get("type").unwrap().as_string();
        self.participants = participants;
        self.id = base.get_name().parse::<u32>().unwrap();
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
