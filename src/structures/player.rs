
use std::{cell::Ref, rc::Rc};

use minijinja::context;
use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::GameObject;

use crate::game_state::GameState;

use super::{renderer::{Cullable, Renderable}, Character, GameObjectDerived, LineageNode, Renderer, Shared};

/// A struct representing a player in the game
pub struct Player {
    pub name: Shared<String>,
    pub id: u32,
    pub character: Option<Shared<Character>>,
    pub lineage: Vec<LineageNode>,
}

/// Gets the lineage of the player and appends it to the lineage vector
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

    fn get_name(&self) -> Shared<String> {
        self.name.clone()
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

impl Player{
    pub fn set_tree_depth(&mut self, depth: usize){
        for node in self.lineage.iter_mut(){
            node.get_character().borrow_mut().set_depth(depth);
        }
    }
}

impl Renderable for Player{
    fn get_context(&self) -> minijinja::Value {
        context!{player=>self}
    }

    fn get_subdir() -> &'static str {
        "."
    }

    fn get_path(&self, path: &str) -> String {
        format!("{}/index.html", path)
    }

    fn get_template() -> &'static str {
        "homeTemplate.html"
    }

    fn render_all(&self, renderer: &mut Renderer){
        println!("Rendering player");
        renderer.render(self);
        for char in self.lineage.iter(){
            println!("Rendering character");
            char.get_character().borrow().render_all(renderer);
        }
    }
}
