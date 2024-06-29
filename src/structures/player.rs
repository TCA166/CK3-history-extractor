use image::{
    codecs::gif::{GifEncoder, Repeat},
    Delay, Frame,
};
use minijinja::context;

use serde::{ser::SerializeStruct, Serialize};

use super::super::{
    display::{Cullable, Localizer, Renderable, RenderableType, Renderer},
    game_object::{GameObject, GameString},
    game_state::GameState,
    types::Wrapper,
};

use super::{Character, FromGameObject, GameId, GameObjectDerived, LineageNode, Shared};
use std::fs::File;

/// A struct representing a player in the game
pub struct Player {
    pub name: GameString,
    pub id: GameId,
    pub character: Option<Shared<Character>>,
    pub lineage: Vec<LineageNode>,
}

/// Gets the lineage of the player and appends it to the lineage vector
fn get_lineage(lineage: &mut Vec<LineageNode>, base: &GameObject, game_state: &mut GameState) {
    let lineage_node = base.get_object_ref("legacy");
    for leg in lineage_node.get_array_iter() {
        let o = leg.as_object().unwrap();
        lineage.push(LineageNode::from_game_object(o, game_state))
    }
}

impl FromGameObject for Player {
    fn from_game_object(base: &GameObject, game_state: &mut GameState) -> Self {
        let mut lineage: Vec<LineageNode> = Vec::new();
        get_lineage(&mut lineage, &base, game_state);
        let key = base.get("character").unwrap().as_id();
        Player {
            name: base.get("name").unwrap().as_string(),
            id: base.get("player").unwrap().as_id(),
            character: Some(game_state.get_character(&key).clone()),
            lineage: lineage,
        }
    }
}

impl GameObjectDerived for Player {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        self.name.clone()
    }
}

impl Serialize for Player {
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

impl Renderable for Player {
    fn get_context(&self) -> minijinja::Value {
        context! {player=>self}
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

    fn render_all(&self, stack: &mut Vec<RenderableType>, renderer: &mut Renderer) {
        renderer.render(self);
        let map = renderer.get_map();
        if map.is_some() {
            //timelapse rendering
            let map = map.unwrap();
            let path = renderer.get_path().to_owned() + "/timelapse.gif";
            let mut file = File::create(&path).unwrap();
            let mut gif_encoder = GifEncoder::new(&mut file);
            for char in self.lineage.iter() {
                /* Note on timelapse:
                Paradox doesn't save any data regarding top level liege changes.
                Not even basic data that would allow us to reconstruct the map through implication.
                We would need something as basic as adding liege changes to history, or even just storing dead character's vassal relations
                I once had an idea that it could be possible to still have a timelapse by looking at dead vassals of the children of chars in lineage
                But that idea got stuck at the recursive step of that algorithm, and even so the result would have NO accuracy
                 */
                let char = char.get_character(); //this variable for no reason other than compiler bitching
                let char = char.get_internal();
                //we get the provinces held by the character and the vassals who died under their reign.
                //This is the closes approximation we can get of changes in the map that are 100% accurate
                let fbytes = map.create_map_buffer(char.get_de_jure_barony_keys(), &[70, 255, 70]);
                //these variables cuz fbytes is moved
                let width = fbytes.width();
                let height = fbytes.height();
                let frame =
                    Frame::from_parts(fbytes, width, height, Delay::from_numer_denom_ms(3000, 1));
                gif_encoder.encode_frame(frame).unwrap();
            }
            gif_encoder.set_repeat(Repeat::Infinite).unwrap();
        }
        for char in self.lineage.iter() {
            char.get_character()
                .get_internal()
                .render_all(stack, renderer);
        let grapher = renderer.get_grapher();
        if grapher.is_some() {
            let last = self.lineage.last().unwrap().get_character();
            grapher.unwrap().create_tree_graph::<Character>(last, true, &format!("{}/line.svg", renderer.get_path()));
        }
        }
    }
}

impl Cullable for Player {
    fn set_depth(&mut self, depth: usize, localization: &Localizer) {
        for node in self.lineage.iter_mut() {
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
