use serde::{ser::SerializeSeq, Serialize, Serializer};

use super::{
    super::{
        display::{Cullable, RenderableType},
        game_data::{Localizable, Localize},
        parser::{GameId, GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError},
        types::{Shared, WrapperMut},
    },
    serialize_ref, Character, DerivedRef, DummyInit, GameObjectDerived,
};

fn serialize_history<S: Serializer>(
    val: &Vec<(
        GameString,
        GameString,
        Option<Shared<Character>>,
        Option<Shared<Character>>,
    )>,
    s: S,
) -> Result<S::Ok, S::Error> {
    let mut seq = s.serialize_seq(Some(val.len()))?;
    for (r#type, date, actor, recipient) in val {
        let actor = if let Some(actor) = actor {
            Some(DerivedRef::from(actor.clone()))
        } else {
            None
        };
        let recipient = if let Some(recipient) = recipient {
            Some(DerivedRef::from(recipient.clone()))
        } else {
            None
        };
        seq.serialize_element(&(r#type, date, actor, recipient))?;
    }
    seq.end()
}

#[derive(Serialize)]
pub struct Artifact {
    id: GameId,
    name: Option<GameString>,
    description: Option<GameString>,
    r#type: Option<GameString>,
    rarity: Option<GameString>,
    quality: u32,
    wealth: u32,
    #[serde(serialize_with = "serialize_ref")]
    owner: Option<Shared<Character>>,
    #[serde(serialize_with = "serialize_history")]
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
        self.name = Some(base.get_string("name")?);
        self.description = Some(base.get_string("description")?);
        self.r#type = Some(base.get_string("type")?);
        self.rarity = Some(base.get_string("rarity")?);
        if let Some(quality_node) = base.get("quality") {
            self.quality = quality_node.as_integer()? as u32;
        } else {
            self.quality = 0;
        }
        if let Some(wealth_node) = base.get("wealth") {
            self.wealth = wealth_node.as_integer()? as u32;
        } else {
            self.wealth = 0;
        }
        self.owner = Some(game_state.get_character(&base.get_game_id("owner")?));
        if let Some(history_node) = base.get("history") {
            let history_node = history_node.as_object()?;
            if history_node.is_empty() {
                return Ok(());
            }
            if let Some(entries_node) = history_node.as_map()?.get("entries") {
                for h in entries_node.as_object()?.as_array()? {
                    let h = h.as_object()?.as_map()?;
                    let r#type = h.get_string("type")?;
                    let date = h.get_string("date")?;
                    let actor = if let Some(actor_node) = h.get("actor") {
                        Some(game_state.get_character(&actor_node.as_id()?))
                    } else {
                        None
                    };
                    let recipient = if let Some(recipient_node) = h.get("recipient") {
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
    fn localize<L: Localize>(&mut self, localization: &mut L) {
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
        let depth = depth - 1;
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
