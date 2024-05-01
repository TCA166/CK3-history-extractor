
use std::{cell::Ref, rc::Rc};

use minijinja::{context, Environment};
use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::GameObject;

use crate::game_state::GameState;

use super::{renderer::{Renderable, Cullable}, Character, GameObjectDerived, LineageNode, Shared};

pub struct Player {
    pub name: Shared<String>,
    pub id: u32,
    pub character: Option<Shared<Character>>,
    pub lineage: Vec<LineageNode>,
}

fn get_lineage(lineage: &mut Vec<LineageNode>, base: &Ref<'_, GameObject>, game_state: &mut GameState){
    let lineage_node = base.get_object_ref("legacy");
    for leg in lineage_node.get_array_iter(){
        let o = leg.as_object_ref().unwrap();
        lineage.push(LineageNode::from_game_object(o, game_state))
    }
}

impl GameObjectDerived for Player {
    fn from_game_object(base: Ref<'_, GameObject>, game_state: &mut GameState) -> Self {
        let mut lineage: Vec<LineageNode> = Vec::new();
        get_lineage(&mut lineage, &base, game_state);
        let key = base.get("character").unwrap().as_string_ref().unwrap();
        Player {
            name: base.get("name").unwrap().as_string(),
            id: base.get("player").unwrap().as_string_ref().unwrap().parse::<u32>().unwrap(),
            character: Some(Rc::from(game_state.get_character(&key).clone())),
            lineage: lineage
        }
    }

    fn dummy(id:u32) -> Self {
        Player {
            name: Rc::new("".to_owned().into()),
            id: id,
            character: None,
            lineage: Vec::new()
        }
    }

    fn init(&mut self, base: Ref<'_, GameObject>, game_state: &mut GameState) {
        let key = base.get("character").unwrap().as_string_ref().unwrap();
        self.character = Some(Rc::from(game_state.get_character(&key).clone()));
        self.id = base.get("player").unwrap().as_string_ref().unwrap().parse::<u32>().unwrap();
        get_lineage(&mut self.lineage, &base, game_state);
    }

    fn get_id(&self) -> u32 {
        self.id
    }
}

impl Serialize for Player{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Player", 4)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("character", &self.character)?;
        state.serialize_field("lineage", &self.lineage)?;
        state.end()
    }
}

impl Renderable for Player{
    fn render(&self, env: &Environment) -> String {
        for char in self.lineage.iter(){
            char.character.as_ref().unwrap().borrow_mut().set_depth(1);
        }
        let ctx = context!{player=>self};
        env.get_template("homeTemplate.html").unwrap().render(&ctx).unwrap()   
    }
}
