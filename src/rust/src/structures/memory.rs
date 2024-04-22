use std::collections::HashMap;
use std::rc::Rc;
use serde::Serialize;
use serde::ser::SerializeStruct;
use super::Character;

pub struct Memory {
    pub date: Rc<String>,
    pub r#type: Rc<String>,
    pub participants: HashMap<Rc<String>, Rc<Character>>,
}

impl Memory {
    pub fn new(date: Rc<String>, r#type: Rc<String>, participants: HashMap<Rc<String>, Rc<Character>>) -> Self {
        Memory {
            date,
            r#type,
            participants,
        }
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
