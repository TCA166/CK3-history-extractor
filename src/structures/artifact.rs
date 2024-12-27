use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        display::{Cullable, Localizable, Localizer, RenderableType},
        parser::{GameId, GameObjectMap, GameState, GameString, KeyError, ParsingError},
        types::{Shared, WrapperMut},
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
    fn init(
        &mut self,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<(), ParsingError> {
        self.name = Some(
            base.get("name")
                .ok_or_else(|| KeyError::MissingKey("name", base.clone()))?
                .as_string()?,
        );
        self.description = Some(base.get("description")?.as_string()?);
        self.r#type = Some(base.get("type")?.as_string()?);
        self.rarity = Some(base.get("rarity")?.as_string()?);
        if let Some(quality_node) = base.get("quality") {
            self.quality = quality_node.as_integer()? as u32;
        } else {
            self.quality = 0;
        }
        if let Ok(wealth_node) = base.get("wealth") {
            self.wealth = wealth_node.as_integer()? as u32;
        } else {
            self.wealth = 0;
        }
        self.owner = Some(game_state.get_character(&base.get("owner")?.as_id()?));
        if let Ok(history_node) = base.get("history") {
            let history_node = history_node.as_object()?.as_map()?;
            if history_node.is_empty() {
                return Ok(());
            }
            if let Ok(entries_node) = history_node.get("entries") {
                for h in entries_node.as_object()?.as_array()? {
                    let h = h.as_object()?.as_map()?;
                    let r#type = h.get("type")?.as_string()?;
                    let date = h.get("date")?.as_string()?;
                    let actor = if let Ok(actor_node) = h.get("actor") {
                        Some(game_state.get_character(&actor_node.as_id()?))
                    } else {
                        None
                    };
                    let recipient = if let Ok(recipient_node) = h.get("recipient") {
                        Some(game_state.get_character(&recipient_node.as_id()?))
                    } else {
                        None
                    };
                    self.history.push((r#type, date, actor, recipient));
                }
            }
        }
        Ok(())
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

impl Localizable for Artifact {
    fn localize(&mut self, localization: &mut Localizer) {
        if let Some(rarity) = &self.rarity {
            self.rarity = Some(localization.localize(rarity));
        }
        if let Some(r#type) = &self.r#type {
            self.r#type = Some(localization.localize(r#type));
        }
    }
}

impl Cullable for Artifact {
    fn get_depth(&self) -> usize {
        self.depth
    }

    fn set_depth(&mut self, depth: usize) {
        if depth <= self.depth {
            return;
        }
        self.depth = depth;
        if let Some(owner) = &self.owner {
            if let Ok(mut owner) = owner.try_get_internal_mut() {
                owner.set_depth(depth);
            }
        }
        for h in self.history.iter_mut() {
            if let Some(actor) = &h.2 {
                if let Ok(mut actor) = actor.try_get_internal_mut() {
                    actor.set_depth(depth);
                }
            }
            if let Some(recipient) = &h.3 {
                if let Ok(mut recipient) = recipient.try_get_internal_mut() {
                    recipient.set_depth(depth);
                }
            }
        }
    }
}

impl Artifact {
    /// Render the characters in the history of the artifact
    pub fn add_ref(&self, stack: &mut Vec<RenderableType>) {
        for h in self.history.iter() {
            if let Some(actor) = &h.2 {
                stack.push(RenderableType::Character(actor.clone()));
            }
            if let Some(recipient) = &h.3 {
                stack.push(RenderableType::Character(recipient.clone()));
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
            let actor = if let Some(actor) = &h.2 {
                Some(DerivedRef::from_derived(actor.clone()))
            } else {
                None
            };
            let recipient = if let Some(recipient) = &h.3 {
                Some(DerivedRef::from_derived(recipient.clone()))
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
