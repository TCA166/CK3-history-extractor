use jomini::common::Date;
use serde::Serialize;

use crate::types::Wrapper;

use super::{
    super::{
        game_data::{GameData, Localizable, LocalizationError, Localize},
        parser::{GameObjectMap, GameObjectMapping, GameState, ParsingError},
        types::GameString,
    },
    Character, EntityRef, FromGameObject, GameObjectDerived, GameRef,
};

#[derive(Serialize)]
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

impl FromGameObject for Artifact {
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let mut artifact = Self {
            name: base.get_string("name")?,
            description: base.get_string("description")?,
            r#type: base.get_string("type")?,
            rarity: base.get_string("rarity")?,
            quality: base
                .get("quality")
                .map_or(Ok(0), |n| n.as_integer().and_then(|v| Ok(v as u32)))?,
            wealth: base
                .get("wealth")
                .map_or(Ok(0), |n| n.as_integer().and_then(|v| Ok(v as u32)))?,
            owner: game_state.get_character(&base.get_game_id("owner")?),
            history: Vec::default(),
        };
        if let Some(history_node) = base.get("history") {
            let history_node = history_node.as_object()?;
            if let Ok(map) = history_node.as_map() {
                if let Some(entries_node) = map.get("entries") {
                    for h in entries_node.as_object()?.as_array()? {
                        let h = h.as_object()?.as_map()?;
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
                        artifact.history.push((
                            h.get_string("type")?,
                            h.get_date("date")?,
                            actor,
                            recipient,
                        ));
                    }
                }
            }
        }
        Ok(artifact)
    }
}

impl GameObjectDerived for Artifact {
    fn get_name(&self) -> GameString {
        self.name.clone()
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
        for h in self.history.iter() {
            if let Some(actor) = &h.2 {
                collection.extend([E::from(actor.clone().into())]);
            }
            if let Some(recipient) = &h.3 {
                collection.extend([E::from(recipient.clone().into())]);
            }
        }
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
    fn localize(&mut self, localization: &GameData) -> Result<(), LocalizationError> {
        self.rarity = localization.localize(&self.rarity)?;
        self.r#type = localization.localize("artifact_".to_string() + self.r#type.as_ref())?;
        self.description = handle_tooltips(&self.description).into();
        self.name = handle_tooltips(&self.name).into();
        Ok(())
    }
}

impl Serialize for GameRef<Artifact> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get_internal().serialize(serializer)
    }
}
