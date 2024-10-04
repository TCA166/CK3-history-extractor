use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        display::{Cullable, Localizer, RenderableType},
        parser::{GameId, GameObjectMap, GameState, GameString},
        types::Shared,
    },
    Character, DerivedRef, DummyInit, GameObjectDerived,
};

pub struct Artifact {
    id: GameId,
    name: Option<GameString>,
    description: Option<GameString>,
    r#type: Option<GameString>,
    rarity: Option<GameString>,
    quality: u32,
    wealth: u32,
    owner: Option<Shared<Character>>,
    history: Vec<(
        GameString,
        GameString,
        Option<Shared<Character>>,
        Option<Shared<Character>>,
    )>,
    depth: usize,
}

impl GameObjectDerived for Artifact {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        self.name.as_ref().unwrap().clone()
    }
}

impl DummyInit for Artifact {
    fn init(&mut self, base: &GameObjectMap, game_state: &mut GameState) {
        self.name = Some(base.get_string_ref("name"));
        self.description = Some(base.get_string_ref("description"));
        self.r#type = Some(base.get_string_ref("type"));
        self.rarity = Some(base.get_string_ref("rarity"));
        let quality_node = base.get("quality");
        if quality_node.is_some() {
            self.quality = quality_node.unwrap().as_string().parse::<u32>().unwrap();
        } else {
            self.quality = 0;
        }
        let wealth_node = base.get("wealth");
        if wealth_node.is_some() {
            self.wealth = wealth_node.unwrap().as_string().parse::<u32>().unwrap();
        } else {
            self.wealth = 0;
        }
        self.owner = Some(game_state.get_character(&base.get("owner").unwrap().as_id()));
        let history_node = base.get("history");
        if history_node.is_some() {
            let history_node = history_node.unwrap().as_object().as_map();
            let entries_node = history_node.get("entries");
            if entries_node.is_some() {
                for h in entries_node.unwrap().as_object().as_array() {
                    let h = h.as_object().as_map();
                    let r#type = h.get_string_ref("type");
                    let date = h.get_string_ref("date");
                    let actor_node = h.get("actor");
                    let actor = if actor_node.is_some() {
                        Some(game_state.get_character(&actor_node.unwrap().as_id()))
                    } else {
                        None
                    };
                    let recipient_node = h.get("recipient");
                    let recipient = if recipient_node.is_some() {
                        Some(game_state.get_character(&recipient_node.unwrap().as_id()))
                    } else {
                        None
                    };
                    self.history.push((r#type, date, actor, recipient));
                }
            }
        }
    }

    fn dummy(id: GameId) -> Self {
        Artifact {
            id,
            name: None,
            description: None,
            r#type: None,
            rarity: None,
            quality: 0,
            wealth: 0,
            owner: None,
            history: Vec::new(),
            depth: 0,
        }
    }
}

impl Cullable for Artifact {
    fn get_depth(&self) -> usize {
        self.depth
    }

    fn set_depth(&mut self, depth: usize, localization: &Localizer) {
        if depth <= self.depth {
            return;
        }
        self.depth = depth;
        if self.owner.is_some() {
            let owner = self.owner.as_ref().unwrap().try_borrow_mut();
            if owner.is_ok() {
                owner.unwrap().set_depth(depth - 1, localization);
            }
        }
        for h in self.history.iter_mut() {
            if h.2.is_some() {
                let actor = h.2.as_ref().unwrap().try_borrow_mut();
                if actor.is_ok() {
                    actor.unwrap().set_depth(depth - 1, localization);
                }
            }
            if h.3.is_some() {
                let recipient = h.3.as_ref().unwrap().try_borrow_mut();
                if recipient.is_ok() {
                    recipient.unwrap().set_depth(depth - 1, localization);
                }
            }
        }
        if self.rarity.is_some() {
            self.rarity = Some(localization.localize(self.rarity.as_ref().unwrap()));
        }
        if self.r#type.is_some() {
            self.r#type = Some(localization.localize(self.r#type.as_ref().unwrap()));
        }
    }
}

impl Artifact {
    /// Render the characters in the history of the artifact
    pub fn render_history(&self, stack: &mut Vec<RenderableType>) {
        for h in self.history.iter() {
            if h.2.is_some() {
                stack.push(RenderableType::Character(h.2.as_ref().unwrap().clone()));
            }
            if h.3.is_some() {
                stack.push(RenderableType::Character(h.3.as_ref().unwrap().clone()));
            }
        }
    }
}

impl Serialize for Artifact {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Artifact", 7)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("type", &self.r#type)?;
        state.serialize_field("rarity", &self.rarity)?;
        state.serialize_field("quality", &self.quality)?;
        state.serialize_field("wealth", &self.wealth)?;
        let mut serialized_history: Vec<(
            GameString,
            GameString,
            Option<DerivedRef<Character>>,
            Option<DerivedRef<Character>>,
        )> = Vec::new();
        for h in self.history.iter() {
            let actor = if h.2.is_some() {
                Some(DerivedRef::from_derived(h.2.as_ref().unwrap().clone()))
            } else {
                None
            };
            let recipient = if h.3.is_some() {
                Some(DerivedRef::from_derived(h.3.as_ref().unwrap().clone()))
            } else {
                None
            };
            serialized_history.push((h.0.clone(), h.1.clone(), actor, recipient));
        }
        state.serialize_field("history", &serialized_history)?;
        state.end()
    }
}

// Comparing implementations so that we can sort artifacts by quality and wealth

impl PartialEq for Artifact {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for Artifact {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let n = self.quality + self.wealth;
        let m = other.quality + other.wealth;
        n.partial_cmp(&m)
    }
}

impl Eq for Artifact {}

impl Ord for Artifact {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
