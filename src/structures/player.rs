use image::{
    codecs::gif::{GifEncoder, Repeat},
    Delay, Frame,
};

use serde::Serialize;

use super::{
    super::{
        display::{Cullable, Grapher, Renderable, RenderableType},
        game_data::{GameMap, Localizable, Localize, MapGenerator},
        jinja_env::H_TEMPLATE_NAME,
        parser::{GameId, GameObjectMap, GameState, GameString, ParsingError},
        types::Wrapper,
    },
    Character, FromGameObject, GameObjectDerived, LineageNode, Shared,
};

use std::{collections::HashSet, fs::File};

const TARGET_COLOR: [u8; 3] = [70, 255, 70];
const SECONDARY_COLOR: [u8; 3] = [255, 255, 70];
const BASE_COLOR: [u8; 3] = [255, 255, 255];

/// A struct representing a player in the game
#[derive(Serialize)]
pub struct Player {
    pub name: GameString,
    pub id: GameId,
    pub character: Option<Shared<Character>>,
    pub lineage: Vec<LineageNode>,
}

impl FromGameObject for Player {
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let mut lineage: Vec<LineageNode> = Vec::new();
        for leg in base.get_object("legacy")?.as_array()? {
            lineage.push(LineageNode::from_game_object(
                leg.as_object()?.as_map()?,
                game_state,
            )?)
        }
        // apparently the player id can be negative?
        let id = base.get_integer("player")? as i32;
        Ok(Player {
            name: base.get_string("name")?,
            id: id as u32,
            character: Some(
                game_state
                    .get_character(&base.get_game_id("character")?)
                    .clone(),
            ),
            lineage: lineage,
        })
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

impl Renderable for Player {
    fn get_subdir() -> &'static str {
        "."
    }

    fn get_path(&self, path: &str) -> String {
        format!("{}/index.html", path)
    }

    fn get_template() -> &'static str {
        H_TEMPLATE_NAME
    }

    fn append_ref(&self, stack: &mut Vec<RenderableType>) {
        for char in self.lineage.iter() {
            stack.push(RenderableType::Character(char.get_character()));
        }
    }

    fn render(
        &self,
        path: &str,
        game_state: &GameState,
        grapher: Option<&Grapher>,
        map: Option<&GameMap>,
    ) {
        if let Some(map) = map {
            //timelapse rendering
            let mut file = File::create(path.to_owned() + "/timelapse.gif").unwrap();
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
                let death_date = char.get_death_date();
                let date = if let Some(death_date) = &death_date {
                    death_date.as_str()
                } else {
                    game_state.get_current_date().unwrap()
                };
                let fbytes = map.create_map_buffer(char.get_barony_keys(true), &TARGET_COLOR, date);
                //these variables cuz fbytes is moved
                let width = fbytes.width();
                let height = fbytes.height();
                let frame =
                    Frame::from_parts(fbytes, width, height, Delay::from_numer_denom_ms(3000, 1));
                gif_encoder.encode_frame(frame).unwrap();
            }
            gif_encoder.set_repeat(Repeat::Infinite).unwrap();
            let mut direct_titles = HashSet::new();
            let mut descendant_title = HashSet::new();
            let first = self.lineage.first().unwrap().get_character();
            let first = first.get_internal();
            let dynasty = first.get_dynasty();
            let dynasty = dynasty.as_ref().unwrap().get_internal();
            let descendants = dynasty.get_founder().get_internal().get_descendants();
            for desc in descendants {
                let desc = desc.get_internal();
                if desc.get_death_date().is_some() {
                    continue;
                }
                let target = if desc
                    .get_dynasty()
                    .map_or(false, |d| d.get_internal().is_same_dynasty(&dynasty))
                {
                    &mut direct_titles
                } else {
                    &mut descendant_title
                };
                for title in desc.get_barony_keys(false) {
                    target.insert(title.clone());
                }
            }
            map.create_map_graph(
                |key: &String| {
                    if direct_titles.contains(key) {
                        return TARGET_COLOR;
                    } else if descendant_title.contains(key) {
                        return SECONDARY_COLOR;
                    } else {
                        return BASE_COLOR;
                    }
                },
                &format!("{}/dynastyMap.png", path),
                vec![
                    ("Dynastic titles".to_string(), TARGET_COLOR),
                    ("Descendant titles".to_string(), SECONDARY_COLOR),
                ],
            );
        }
        if let Some(grapher) = grapher {
            let last = self.lineage.last().unwrap().get_character();
            grapher.create_tree_graph::<Character>(last, true, &format!("{}/line.svg", path));
        }
    }
}

impl Localizable for Player {
    fn localize<L: Localize>(&mut self, localization: &mut L) {
        for node in self.lineage.iter_mut() {
            node.localize(localization);
        }
    }
}

impl Cullable for Player {
    fn set_depth(&mut self, depth: usize) {
        for node in self.lineage.iter_mut() {
            node.set_depth(depth);
        }
    }

    fn get_depth(&self) -> usize {
        0
    }

    fn is_ok(&self) -> bool {
        true
    }
}
