use std::path::Path;

use jomini::common::Date;
use serde::Serialize;

use super::{
    super::{
        display::{Grapher, ProceduralPath, Renderable, TreeNode},
        game_data::{GameData, Localizable, LocalizationError, Localize, MapGenerator},
        jinja_env::CUL_TEMPLATE_NAME,
        parser::{GameId, GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError},
        types::{Wrapper, WrapperMut},
    },
    EntityRef, FromGameObject, GameObjectDerived, GameObjectEntity, GameRef, Title,
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

impl FromGameObject for Culture {
    fn from_game_object(
        id: GameId,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let mut culture = Self {
            name: base.get_string("name")?,
            //this is possible, shoutout u/Kinc4id
            ethos: base.get("ethos").map(|n| n.as_string()).transpose()?,
            heritage: base.get_string("heritage")?,
            martial: base.get_string("martial_custom")?,
            date: base.get("created").map(|n| n.as_date()).transpose()?,
            language: base.get_string("language")?,
            traditions: base
                .get("traditions")
                .map(|n| n.as_object().and_then(|obj| obj.as_array()))
                .transpose()?
                .map_or(Vec::new(), |n| {
                    n.iter().filter_map(|t| t.as_string().ok()).collect()
                }),
            children: Vec::new(),
            parents: Vec::new(),
        };
        if let Some(parents_obj) = base.get("parents") {
            for p in parents_obj.as_object()?.as_array()? {
                let parent = game_state.get_culture(&p.as_id()?).clone();
                if let Ok(mut r) = parent.try_get_internal_mut() {
                    r.register_child(game_state.get_culture(&id));
                }
                culture.parents.push(parent.clone());
            }
        }
        return Ok(culture);
    }
}

impl GameObjectDerived for Culture {
    fn get_name(&self) -> GameString {
        self.name.clone()
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
        for p in &self.parents {
            collection.extend([E::from(p.clone().into())]);
        }
        for c in &self.children {
            collection.extend([E::from(c.clone().into())]);
        }
    }
}

impl TreeNode<Vec<GameRef<Culture>>> for Culture {
    fn get_children(&self) -> Option<Vec<GameRef<Culture>>> {
        if self.children.is_empty() {
            return None;
        }
        Some(self.children.clone())
    }

    fn get_parent(&self) -> Option<Vec<GameRef<Culture>>> {
        if self.parents.is_empty() {
            return None;
        }
        Some(self.parents.clone())
    }

    fn get_class(&self) -> Option<GameString> {
        Some(self.heritage.clone())
    }
}

impl Culture {
    pub fn register_child(&mut self, child: GameRef<Culture>) {
        self.children.push(child);
    }
}

impl ProceduralPath for Culture {
    fn get_subdir() -> &'static str {
        "cultures"
    }
}

impl Renderable for GameObjectEntity<Culture> {
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
            let mut path = path.join(Culture::get_subdir());
            path.push(format!("{}.svg", self.id));
            grapher.create_culture_graph(self.id, &path);
        }
        if let Some(map) = data.get_map() {
            let filter = |title: &Title| {
                if title.get_key().starts_with("c_") {
                    if let Some(c_culture) = title.get_culture() {
                        return c_culture.get_internal().id == self.id;
                    }
                }
                return false;
            };
            let keys = game_state.get_baronies_of_counties(filter);
            if !keys.is_empty() {
                let mut path = path.join(Culture::get_subdir());
                path.push(format!("{}.png", self.id));
                let mut culture_map = map.create_map_flat(keys, [70, 255, 70]);
                culture_map.draw_text(format!("Map of the {} culture", &self.name));
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
        self.name = localization.localize(&self.name)?;
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
