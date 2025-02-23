use std::{cell::Ref, path::Path};

use jomini::common::Date;
use serde::Serialize;

use super::{
    super::{
        display::{Grapher, Renderable, TreeNode},
        game_data::{GameData, Localizable, LocalizationError, Localize, MapGenerator},
        jinja_env::CUL_TEMPLATE_NAME,
        parser::{GameId, GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError},
        types::{OneOrMany, Wrapper, WrapperMut},
    },
    DummyInit, GameObjectDerived, GameObjectDerivedType, Shared, Title,
};

/// A struct representing a culture in the game
#[derive(Serialize, Debug)]
pub struct Culture {
    id: GameId,
    name: Option<GameString>,
    ethos: Option<GameString>,
    heritage: Option<GameString>,
    martial: Option<GameString>,
    date: Option<Date>,
    children: Vec<Shared<Culture>>,
    parents: Vec<Shared<Culture>>,
    traditions: Vec<GameString>,
    language: Option<GameString>,
    // TODO innovations
}

impl DummyInit for Culture {
    fn dummy(id: GameId) -> Self {
        Culture {
            name: None,
            ethos: None,
            heritage: None,
            martial: None,
            date: None,
            parents: Vec::new(),
            children: Vec::new(),
            traditions: Vec::new(),
            language: None,
            id: id,
        }
    }

    fn init(
        &mut self,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<(), ParsingError> {
        if let Some(parents_obj) = base.get("parents") {
            for p in parents_obj.as_object()?.as_array()? {
                let parent = game_state.get_culture(&p.as_id()?).clone();
                if let Ok(mut r) = parent.try_get_internal_mut() {
                    r.register_child(game_state.get_culture(&self.id));
                }
                self.parents.push(parent.clone());
            }
        }
        if let Some(traditions_obj) = base.get("traditions") {
            for t in traditions_obj.as_object()?.as_array()? {
                self.traditions.push(t.as_string()?);
            }
        }
        self.name = Some(base.get_string("name")?);
        if let Some(eth) = base.get("ethos") {
            //this is possible, shoutout u/Kinc4id
            self.ethos = Some(eth.as_string()?);
        }
        self.heritage = Some(base.get_string("heritage")?);
        self.martial = Some(base.get_string("martial_custom")?);
        if let Some(node) = base.get("created") {
            self.date = Some(node.as_date()?);
        }
        self.language = Some(base.get_string("language")?);
        Ok(())
    }
}

impl GameObjectDerived for Culture {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> Option<GameString> {
        self.name.as_ref().map(|x| x.clone())
    }

    fn get_references<E: From<GameObjectDerivedType>, C: Extend<E>>(&self, collection: &mut C) {
        for p in &self.parents {
            collection.extend([E::from(p.clone().into())]);
        }
        for c in &self.children {
            collection.extend([E::from(c.clone().into())]);
        }
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

impl Renderable for Culture {
    fn get_template() -> &'static str {
        CUL_TEMPLATE_NAME
    }

    fn get_subdir() -> &'static str {
        "cultures"
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
        self.name = Some(localization.localize(self.name.as_ref().unwrap())?);
        if let Some(eth) = &self.ethos {
            self.ethos = Some(localization.localize(eth.to_string() + "_name")?);
        }
        if let Some(heritage) = &self.heritage {
            self.heritage = Some(localization.localize(heritage.to_string() + "_name")?);
        }
        if let Some(martial) = &self.martial {
            self.martial = Some(localization.localize(martial.to_string() + "_name")?);
        }
        if let Some(language) = &self.language {
            self.language = Some(localization.localize(language.to_string() + "_name")?);
        }
        for t in &mut self.traditions {
            *t = localization.localize(t.to_string() + "_name")?;
        }
        Ok(())
    }
}
