use serde::Serialize;

use super::{
    super::{
        display::{Cullable, Grapher, Renderable, RenderableType, TreeNode},
        game_data::{GameData, Localizable, Localize, MapGenerator},
        jinja_env::CUL_TEMPLATE_NAME,
        parser::{GameId, GameObjectMap, GameState, GameString, ParsingError},
        types::{OneOrMany, RefOrRaw, Wrapper, WrapperMut},
    },
    serialize_array_ref, DummyInit, GameObjectDerived, Shared, Title,
};

/// A struct representing a culture in the game
#[derive(Serialize)]
pub struct Culture {
    id: GameId,
    name: Option<GameString>,
    ethos: Option<GameString>,
    heritage: Option<GameString>,
    martial: Option<GameString>,
    date: Option<GameString>,
    #[serde(serialize_with = "serialize_array_ref")]
    children: Vec<Shared<Culture>>,
    #[serde(serialize_with = "serialize_array_ref")]
    parents: Vec<Shared<Culture>>,
    traditions: Vec<GameString>,
    language: Option<GameString>,
    depth: usize,
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
            depth: 0,
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
            self.date = Some(node.as_string()?);
        }
        self.language = Some(base.get_string("language")?);
        Ok(())
    }
}

impl GameObjectDerived for Culture {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        self.name.as_ref().unwrap().clone()
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
        path: &str,
        game_state: &GameState,
        grapher: Option<&Grapher>,
        data: &GameData,
    ) {
        if let Some(grapher) = grapher {
            let path = format!("{}/{}/{}.svg", path, Self::get_subdir(), self.id);
            grapher.create_culture_graph(self.id, &path);
        }
        if let Some(map) = data.get_map() {
            let filter = |title: &RefOrRaw<Title>| {
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
                let path = format!("{}/{}/{}.png", path, Self::get_subdir(), self.id);
                map.create_map_file(
                    keys,
                    &[70, 255, 70],
                    &path,
                    &format!("Map of the {} culture", &self.name.as_ref().unwrap()),
                );
            }
        }
    }

    fn append_ref(&self, stack: &mut Vec<RenderableType>) {
        for p in &self.parents {
            stack.push(RenderableType::Culture(p.clone()));
        }
        for c in &self.children {
            stack.push(RenderableType::Culture(c.clone()));
        }
    }
}

impl Localizable for Culture {
    fn localize<L: Localize>(&mut self, localization: &mut L) {
        self.name = Some(localization.localize(self.name.as_ref().unwrap().as_str()));
        self.ethos = Some(localization.localize(self.ethos.as_ref().unwrap().as_str()));
        self.heritage = Some(localization.localize(self.heritage.as_ref().unwrap().as_str()));
        self.martial = Some(localization.localize(self.martial.as_ref().unwrap().as_str()));
        self.language = Some(localization.localize(self.language.as_ref().unwrap().as_str()));
        for t in &mut self.traditions {
            *t = localization.localize(t.as_str());
        }
    }
}

impl Cullable for Culture {
    fn get_depth(&self) -> usize {
        self.depth
    }

    fn set_depth(&mut self, depth: usize) {
        if depth <= self.depth {
            return;
        }
        self.depth = depth;
        let depth = depth - 1;
        for p in &self.parents {
            if let Ok(mut r) = p.try_get_internal_mut() {
                r.set_depth(depth);
            }
        }
        for c in &self.children {
            if let Ok(mut r) = c.try_get_internal_mut() {
                r.set_depth(depth);
            }
        }
    }
}
