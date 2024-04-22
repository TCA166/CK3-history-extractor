use std::collections::HashMap;

use serde::Serialize;
use serde::ser::SerializeStruct;

use super::Character;


pub struct Memory<'a> {
    date: &'a String,
    r#type: &'a String,
    participants: HashMap<&'a String, &'a Character<'a>>
}

impl Serialize for Memory<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Memory", 3)?;
        state.serialize_field("date", self.date)?;
        state.serialize_field("type", self.r#type)?;
        state.serialize_field("participants", &self.participants)?;
        state.end()
    }
}
