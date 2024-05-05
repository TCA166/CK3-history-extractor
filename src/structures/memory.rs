use std::cell::{RefCell, Ref};
use std::rc::Rc;
use serde::Serialize;
use serde::ser::SerializeStruct;
use super::{Character, Cullable, GameObjectDerived, Shared};
use crate::game_object::GameObject;
use crate::game_state::GameState;

pub struct Memory {
    id: u32,
    date: Shared<String>,
    r#type: Shared<String>,
    participants: Vec<(String, Shared<Character>)>,
    depth: usize
}

fn get_participants(participants:&mut Vec<(String, Shared<Character>)>, base:&Ref<'_, GameObject>, game_state:&mut GameState){
    let participants_node = base.get("participants");
    if participants_node.is_some(){
        for part in participants_node.unwrap().as_object_ref().unwrap().get_obj_iter(){
            participants.push((part.0.clone(), game_state.get_character(part.1.as_string().borrow().as_str()).clone()));
        }
    }
}

impl GameObjectDerived for Memory {
    fn from_game_object(base: Ref<'_, GameObject>, game_state: &mut GameState) -> Self {
        let mut participants = Vec::new();
        get_participants(&mut participants, &base, game_state);
        Memory{
            date: base.get("creation_date").unwrap().as_string(),
            r#type: base.get("type").unwrap().as_string(),
            participants: participants,
            id: base.get_name().parse::<u32>().unwrap(),
            depth: 0
        }
    }

    fn dummy(id:u32) -> Self {
        Memory{
            date: Rc::new(RefCell::new("".to_owned())),
            r#type: Rc::new(RefCell::new("".to_owned())),
            participants: Vec::new(),
            id: id,
            depth: 0
        }
    }

    fn init(&mut self, base: Ref<'_, GameObject>, game_state: &mut GameState) {
        self.date = base.get("date").unwrap().as_string();
        self.r#type = base.get("type").unwrap().as_string();
        get_participants(&mut self.participants, &base, game_state);
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn get_name(&self) -> Shared<String> {
        self.r#type.clone()
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

impl Cullable for Memory {
    fn set_depth(&mut self, depth:usize){
        if depth <= self.depth || depth == 0{
            return;
        }
        self.depth = depth;
        for part in self.participants.iter_mut(){
            part.1.borrow_mut().set_depth(depth - 1);
        }
    }

    fn get_depth(&self) -> usize{
        self.depth
    }
}
