use std::{cell::Ref, path::Path};

use jomini::common::Date;
use serde::Serialize;

use super::{
    super::{
        display::{Grapher, ProceduralPath, Renderable, TreeNode},
        game_data::{GameData, Localizable, LocalizationError, Localize, MapGenerator},
        jinja_env::CUL_TEMPLATE_NAME,
        parser::{GameId, GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError},
        types::{OneOrMany, Wrapper, WrapperMut},
    },
    GameObjectDerived, GameObjectEntity, GameRef, Shared, Title,
};

/// A struct representing a culture in the game
#[derive(Serialize, Debug)]
pub struct Culture {
    name: GameString,
    ethos: Option<GameString>,
    heritage: GameString,
    martial: GameString,
    date: Option<Date>,
    children: Vec<GameRef<Culture>>,
    parents: Vec<GameRef<Culture>>,
    traditions: Vec<GameString>,
    language: GameString,
    // TODO innovations
}

impl GameObjectDerived for Culture {
    fn get_name(&self) -> GameString {
        self.name.clone()
    }

    fn get_references<T: GameObjectDerived, E: From<GameObjectEntity<T>>, C: Extend<E>>(
        &self,
        collection: &mut C,
    ) {
        for p in &self.parents {
            collection.extend([E::from(p.clone().into())]);
        }
        for c in &self.children {
            collection.extend([E::from(c.clone().into())]);
        }
    }

    fn new(
        id: GameId,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        Ok(Self {
            name: base.get_string("name")?,
            //this is possible, shoutout u/Kinc4id
            ethos: base.get("ethos").map(|n| n.as_string()?),
            heritage: base.get_string("heritage")?,
            martial: base.get_string("martial_custom")?,
            date: base.get("created").map(|n| n.as_date()?),
            language: base.get_string("language")?,
            traditions: base.get("traditions").map_or(Vec::new(), |n| {
                n.as_object()?
                    .as_array()?
                    .iter()
                    .map(|t| t.as_string()?)
                    .collect()
            }),
            children: Vec::new(),
            parents: base.get("parents").map_or(Vec::new(), |n| {
                n.as_object()?
                    .as_array()?
                    .iter()
                    .map(|p| {
                        let parent = game_state.get_culture(&p.as_id()?);
                        if let Ok(mut r) = parent.try_get_internal_mut() {
                            r.register_child(game_state.get_culture(&id));
                        }
                        parent
                    })
                    .collect()
            }),
        })
    }
}

impl TreeNode for Culture {
    fn get_children(&self) -> Option<OneOrMany<Self>> {
        if self.children.is_empty() {
            return None;
        }
        Some(OneOrMany::Many(&self.children))
    }

    fn get_parent(&self) -> Option<OneOrMany<Self>> {
        if self.parents.is_empty() {
            return None;
        }
        Some(OneOrMany::Many(&self.parents))
    }

    fn get_class(&self) -> Option<GameString> {
        if let Some(heritage) = &self.heritage {
            return Some(heritage.clone());
        } else {
            return None;
        }
    }
}

impl Culture {
    pub fn register_child(&mut self, child: Shared<Culture>) {
        self.children.push(child);
    }
}

impl ProceduralPath for Culture {
    fn get_subdir() -> &'static str {
        "cultures"
    }
}

impl Renderable for Culture {
    fn get_template() -> &'static str {
        CUL_TEMPLATE_NAME
    }

    fn render(
        &self,
        path: &Path,
        game_state: &GameState,
        grapher: Option<&Grapher>,
        data: &GameData,
    ) {
        if let Some(grapher) = grapher {
            let mut path = path.join(Self::get_subdir());
            path.push(format!("{}.svg", self.id));
            grapher.create_culture_graph(self.id, &path);
        }
        if let Some(map) = data.get_map() {
            let filter = |title: Ref<Title>| {
                let key = title.get_key();
                if key.is_none() {
                    return false;
                }
                if key.as_ref().unwrap().starts_with("c_") {
                    if let Some(c_culture) = title.get_culture() {
                        return c_culture.get_internal().id == self.id;
                    }
                }
                return false;
            };
            let keys = game_state.get_baronies_of_counties(filter);
            if !keys.is_empty() {
                let mut path = path.join(Self::get_subdir());
                path.push(format!("{}.png", self.id));
                let mut culture_map = map.create_map_flat(keys, [70, 255, 70]);
                culture_map.draw_text(format!(
                    "Map of the {} culture",
                    &self.name.as_ref().unwrap()
                ));
                culture_map.save(&path);
            }
        }
    }
}

impl Localizable for Culture {
    fn localize<L: Localize<GameString>>(
        &mut self,
        localization: &mut L,
    ) -> Result<(), LocalizationError> {
        self.name = localization.localize(self.name)?;
        if let Some(eth) = &self.ethos {
            self.ethos = Some(localization.localize(eth.to_string() + "_name")?);
        }
        self.heritage = localization.localize(self.heritage.to_string() + "_name")?;
        self.martial = localization.localize(self.martial.to_string() + "_name")?;
        self.language = localization.localize(self.language.to_string() + "_name")?;
        for t in &mut self.traditions {
            *t = localization.localize(t.to_string() + "_name")?;
        }
        Ok(())
    }
}
