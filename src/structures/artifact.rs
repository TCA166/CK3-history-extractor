use jomini::common::Date;
use serde::{Serialize, Serializer};

use super::{
    super::{
        game_data::{Localizable, LocalizationError, Localize},
        parser::{GameId, GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError},
        types::Shared,
    },
    Character, GameObjectDerived, GameObjectEntity, GameRef, Wrapper,
};

#[derive(Serialize, Debug)]
pub struct Artifact {
    name: GameString,
    description: GameString,
    r#type: GameString,
    rarity: GameString,
    quality: u32,
    wealth: u32,
    owner: GameRef<Character>,
    history: Vec<(
        GameString,
        Date,
        Option<GameRef<Character>>,
        Option<GameRef<Character>>,
    )>,
}

impl GameObjectDerived for Artifact {
    fn get_name(&self) -> GameString {
        self.name.clone()
    }

    fn get_references<T: GameObjectDerived, E: From<GameObjectEntity<T>>, C: Extend<E>>(
        &self,
        collection: &mut C,
    ) {
        for h in self.history.iter() {
            if let Some(actor) = &h.2 {
                collection.extend([E::from(actor.clone().into())]);
            }
            if let Some(recipient) = &h.3 {
                collection.extend([E::from(recipient.clone().into())]);
            }
        }
    }

    fn new(
        id: GameId,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        Ok(Self {
            name: base.get_string("name")?,
            description: base.get_string("description")?,
            r#type: base.get_string("type")?,
            rarity: base.get_string("rarity")?,
            quality: base.get("quality").map_or(0, |n| n.as_integer()? as u32),
            wealth: base.get("wealth").map_or(0, |n| n.as_integer()? as u32),
            owner: game_state.get_character(&base.get_game_id("owner")?),
            history: if let Some(history_node) = base.get("history") {
                let history_node = history_node.as_object()?;
                if history_node.is_empty() {
                    Vec::new()
                } else {
                    history_node
                        .get("entries")
                        .map_or(Vec::new(), |entries_node| {
                            entries_node
                                .as_object()?
                                .as_array()?
                                .iter()
                                .map(|h| {
                                    let h = h.as_object()?.as_map()?;
                                    let r#type = h.get_string("type")?;
                                    let date = h.get_date("date")?;
                                    let actor = if let Some(actor_node) = h.get("actor") {
                                        Some(game_state.get_character(&actor_node.as_id()?))
                                    } else {
                                        None
                                    };
                                    let recipient = if let Some(recipient_node) = h.get("recipient")
                                    {
                                        Some(game_state.get_character(&recipient_node.as_id()?))
                                    } else {
                                        None
                                    };
                                    Ok((r#type, date, actor, recipient))
                                })
                                .collect::<Result<Vec<_>, ParsingError>>()?
                        })
                }
            } else {
                Vec::new()
            },
        })
    }
}

fn handle_tooltips(text: &GameString) -> String {
    let mut result = String::new();
    let mut in_tooltip = false;
    let mut in_tooltip_text = false;
    for c in text.chars() {
        match c {
            '\x15' => {
                // NAK character precedes a tooltip
                in_tooltip = true;
                in_tooltip_text = false;
            }
            ' ' => {
                if in_tooltip && !in_tooltip_text {
                    in_tooltip_text = true;
                } else {
                    result.push(c);
                }
            }
            '!' => {
                // NAK! character ends a tooltip? I think?
                if in_tooltip {
                    in_tooltip = false;
                    in_tooltip_text = false;
                } else {
                    result.push(c);
                }
            }
            _ => {
                if !in_tooltip || in_tooltip_text {
                    result.push(c);
                }
            }
        }
    }
    return result;
}

impl Localizable for Artifact {
    fn localize<L: Localize<GameString>>(
        &mut self,
        localization: &mut L,
    ) -> Result<(), LocalizationError> {
        if let Some(rarity) = &self.rarity {
            self.rarity = localization.localize(rarity)?;
        }
        if let Some(r#type) = &self.r#type {
            self.r#type = localization.localize("artifact_".to_string() + r#type)?;
        }
        if let Some(desc) = &self.description {
            self.description = handle_tooltips(desc).into();
        }
        if let Some(name) = &self.name {
            self.name = handle_tooltips(name).into();
        }
        Ok(())
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

impl Serialize for Shared<Artifact> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.get_internal().serialize(serializer)
    }
}
