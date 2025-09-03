use super::{
    super::{
        game_data::{GameData, Localizable, LocalizationError},
        parser::{GameObjectMap, GameObjectMapping, GameState, ParsingError},
        types::GameString,
    },
    Character, EntityRef, FromGameObject, GameObjectDerived, GameRef, LineageNode,
};

/// A struct representing a player in the game
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Player {
    name: GameString,
    character: Option<GameRef<Character>>,
    lineage: Vec<LineageNode>,
}

impl FromGameObject for Player {
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let mut player = Self {
            name: base.get_string("name")?,
            character: Some(
                game_state
                    .get_character(&base.get_game_id("character")?)
                    .clone(),
            ),
            lineage: Vec::new(),
        };
        for leg in base.get_object("legacy")?.as_array()? {
            player.lineage.push(LineageNode::from_game_object(
                leg.as_object()?.as_map()?,
                game_state,
            )?)
        }
        Ok(player)
    }
}

impl GameObjectDerived for Player {
    fn get_name(&self) -> GameString {
        self.name.clone()
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
        collection.extend([E::from(self.character.as_ref().unwrap().clone().into())]);
        for node in self.lineage.iter() {
            collection.extend([E::from(node.get_character().clone().into())]);
        }
    }
}

impl Localizable for Player {
    fn localize(&mut self, localization: &GameData) -> Result<(), LocalizationError> {
        for node in self.lineage.iter_mut() {
            node.localize(localization)?;
        }
        Ok(())
    }
}

#[cfg(feature = "display")]
mod display {
    use super::super::super::{
        display::{GetPath, Grapher, Renderable},
        game_data::{MapGenerator, MapImage},
        types::Wrapper,
    };
    use super::*;

    use std::{
        collections::HashSet,
        fs::File,
        ops::Deref,
        path::{Path, PathBuf},
    };

    use image::{
        buffer::ConvertBuffer,
        codecs::gif::{GifEncoder, Repeat},
        Delay, Frame, Rgb,
    };
    use jomini::common::PdsDate;

    const TARGET_COLOR: Rgb<u8> = Rgb([70, 255, 70]);
    const SECONDARY_COLOR: Rgb<u8> = Rgb([255, 255, 70]);
    const BASE_COLOR: Rgb<u8> = Rgb([255, 255, 255]);

    impl GetPath for Player {
        fn get_path(&self, path: &Path) -> PathBuf {
            path.join("index.html")
        }
    }

    impl Renderable for Player {
        const TEMPLATE_NAME: &'static str = "homeTemplate";

        fn render(
            &self,
            path: &Path,
            game_state: &GameState,
            grapher: Option<&Grapher>,
            data: &GameData,
        ) {
            if let Some(map) = data.get_map() {
                //timelapse rendering
                let mut file = File::create(path.join("timelapse.gif")).unwrap();
                let mut gif_encoder = GifEncoder::new(&mut file);
                for char in self.lineage.iter() {
                    /* Note on timelapse:
                    Paradox doesn't save any data regarding top level liege changes.
                    Not even basic data that would allow us to reconstruct the map through implication.
                    We would need something as basic as adding liege changes to history, or even just storing dead character's vassal relations
                    I once had an idea that it could be possible to still have a timelapse by looking at dead vassals of the children of chars in lineage
                    But that idea got stuck at the recursive step of that algorithm, and even so the result would have NO accuracy
                     */
                    if let Some(char) = char.get_character().get_internal().inner() {
                        //we get the provinces held by the character and the vassals who died under their reign.
                        //This is the closes approximation we can get of changes in the map that are 100% accurate
                        let death_date = char.get_death_date();
                        let date = if let Some(death_date) = &death_date {
                            death_date.iso_8601()
                        } else {
                            game_state.get_current_date().unwrap().iso_8601()
                        };
                        let mut barony_map =
                            map.create_map_flat(char.get_barony_keys(true), TARGET_COLOR);
                        barony_map.draw_text(date.to_string());
                        let fbytes = barony_map.convert();
                        //these variables cuz fbytes is moved
                        let width = fbytes.width();
                        let height = fbytes.height();
                        let frame = Frame::from_parts(
                            fbytes,
                            width,
                            height,
                            Delay::from_numer_denom_ms(3000, 1),
                        );
                        gif_encoder.encode_frame(frame).unwrap();
                    }
                }
                gif_encoder.set_repeat(Repeat::Infinite).unwrap();
                let mut direct_titles = HashSet::new();
                let mut descendant_title = HashSet::new();
                let first = self.lineage.first().unwrap().get_character();
                if let Some(first) = first.get_internal().inner() {
                    let dynasty = first.get_house();
                    let dynasty = dynasty.as_ref().unwrap().get_internal();
                    for desc in dynasty
                        .inner()
                        .unwrap()
                        .get_founder()
                        .get_internal()
                        .inner()
                        .unwrap()
                        .get_descendants()
                    {
                        if let Some(desc) = desc.get_internal().inner() {
                            if desc.get_death_date().is_some() {
                                continue;
                            }
                            let target = if desc.get_house().map_or(false, |d| {
                                d.get_internal()
                                    .inner()
                                    .unwrap()
                                    .get_dynasty()
                                    .get_internal()
                                    .deref()
                                    == dynasty
                                        .inner()
                                        .unwrap()
                                        .get_dynasty()
                                        .get_internal()
                                        .deref()
                            }) {
                                &mut direct_titles
                            } else {
                                &mut descendant_title
                            };
                            for title in desc.get_barony_keys(false) {
                                target.insert(title.clone());
                            }
                        }
                    }
                }
                let mut dynasty_map = map.create_map::<_, _, Vec<GameString>>(
                    |key: &str| {
                        if direct_titles.contains(key) {
                            return TARGET_COLOR;
                        } else if descendant_title.contains(key) {
                            return SECONDARY_COLOR;
                        } else {
                            return BASE_COLOR;
                        }
                    },
                    None,
                );
                dynasty_map.draw_legend([
                    ("Dynastic titles".to_string(), TARGET_COLOR),
                    ("Descendant titles".to_string(), SECONDARY_COLOR),
                ]);
                dynasty_map.save_in_thread(path.join("dynastyMap.png"));
            }
            if let Some(grapher) = grapher {
                let last = self.lineage.last().unwrap().get_character();
                grapher.create_tree_graph(last, true, &path.join("line.svg"));
            }
        }
    }
}
