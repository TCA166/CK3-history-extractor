use minijinja::context;
use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::{GameObject, GameString};

use crate::game_state::GameState;

use crate::localizer::Localizer;
use crate::types::Wrapper;

use super::{renderer::{Cullable, Renderable}, Character, GameId, GameObjectDerived, LineageNode, Renderer, Shared};

/// A struct representing a player in the game
pub struct Player {
    pub name: GameString,
    pub id: GameId,
    pub character: Option<Shared<Character>>,
    pub lineage: Vec<LineageNode>,
}

/// Gets the lineage of the player and appends it to the lineage vector
fn get_lineage(lineage: &mut Vec<LineageNode>, base: &GameObject, game_state: &mut GameState){
    let lineage_node = base.get_object_ref("legacy");
    for leg in lineage_node.get_array_iter(){
        let o = leg.as_object().unwrap();
        lineage.push(LineageNode::from_game_object(o, game_state))
    }
}

impl GameObjectDerived for Player {
    fn from_game_object(base: &GameObject, game_state: &mut GameState) -> Self {
        let mut lineage: Vec<LineageNode> = Vec::new();
        get_lineage(&mut lineage, &base, game_state);
        let key = base.get("character").unwrap().as_id();
        Player {
            name: base.get("name").unwrap().as_string(),
            id: base.get("player").unwrap().as_id(),
            character: Some(game_state.get_character(&key).clone()),
            lineage: lineage
        }
    }

    fn dummy(id:GameId) -> Self {
        Player {
            name: GameString::wrap("".to_owned().into()),
            id: id,
            character: None,
            lineage: Vec::new()
        }
    }

    fn init(&mut self, base: &GameObject, game_state: &mut GameState) {
        let key = base.get("character").unwrap().as_id();
        self.character = Some(game_state.get_character(&key).clone());
        self.id = base.get("player").unwrap().as_string().parse::<GameId>().unwrap();
        get_lineage(&mut self.lineage, &base, game_state);
    }

    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
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
        renderer.render(self);
        for char in self.lineage.iter(){
            char.get_character().get_internal().render_all(renderer);
        }
    }
}

impl Cullable for Player{
    fn set_depth(&mut self, depth: usize, localization:&Localizer){
        for node in self.lineage.iter_mut(){
            node.set_depth(depth, localization);
        }
    }

    fn get_depth(&self) -> usize {
        0
    }

    fn is_ok(&self) -> bool {
        true
    }
}
