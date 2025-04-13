use jomini::common::Date;
use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        game_data::{GameData, Localizable, LocalizationError, Localize},
        parser::{GameObjectMap, GameObjectMapping, GameState, ParsingError, SaveFileValue},
        types::{GameString, Wrapper},
    },
    Character, EntityRef, FromGameObject, GameObjectDerived, GameRef,
};

/// A struct representing a lineage node in the game
pub struct LineageNode {
    character: GameRef<Character>,
    date: Date,
    score: i32,
    prestige: i32,
    piety: i32,
    dread: f32,
    lifestyle: Option<GameString>,
    perks: Vec<GameString>, //in older CK3 version this was a list, guess it no longer is
}

impl LineageNode {
    /// Gets the character associated with the lineage node
    pub fn get_character(&self) -> GameRef<Character> {
        self.character.clone()
    }
}

impl FromGameObject for LineageNode {
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let mut perks = Vec::new();
        if let Some(perks_node) = base.get("perk") {
            if let SaveFileValue::Object(o) = perks_node {
                for perk in o.as_array()? {
                    perks.push(perk.as_string()?)
                }
            } else {
                perks.push(perks_node.as_string()?);
            }
        }
        Ok(LineageNode {
            character: game_state.get_character(&base.get_game_id("character")?),
            date: base.get_date("date")?,
            score: if let Some(score_node) = base.get("score") {
                score_node.as_integer()? as i32
            } else {
                0
            },
            prestige: if let Some(prestige_node) = base.get("prestige") {
                prestige_node.as_integer()? as i32
            } else {
                0
            },
            piety: if let Some(piety_node) = base.get("piety") {
                piety_node.as_integer()? as i32
            } else {
                0
            },
            dread: if let Some(dread_node) = base.get("dread") {
                dread_node.as_real()? as f32
            } else {
                0.0
            },
            lifestyle: if let Some(lifestyle_node) = base.get("lifestyle") {
                Some(lifestyle_node.as_string()?)
            } else {
                None
            },
            perks: perks,
        })
    }
}

impl GameObjectDerived for LineageNode {
    fn get_name(&self) -> GameString {
        self.character
            .get_internal()
            .inner()
            .expect("Character in lineage must be initialized")
            .get_name()
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
        collection.extend([E::from(self.character.clone().into())]);
    }
}

impl Localizable for LineageNode {
    fn localize(&mut self, localization: &GameData) -> Result<(), LocalizationError> {
        if let Some(lifestyle) = &self.lifestyle {
            self.lifestyle = Some(localization.localize(lifestyle.to_string() + "_name")?);
        }
        for perk in self.perks.iter_mut() {
            let mut perk_key = perk.to_string();
            if perk_key == "family_man_perk" {
                perk_key += if self
                    .character
                    .get_internal()
                    .inner()
                    .expect("Character in lineage must be initialized")
                    .get_female()
                {
                    "_female_name"
                } else {
                    "_male_name"
                }
            } else {
                perk_key += "_name";
            }
            *perk = localization.localize(perk_key)?;
        }
        Ok(())
    }
}

impl Serialize for LineageNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("LineageNode", 8)?;
        state.serialize_field("character", &*self.character.get_internal())?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("score", &self.score)?;
        state.serialize_field("prestige", &self.prestige)?;
        state.serialize_field("piety", &self.piety)?;
        state.serialize_field("dread", &self.dread)?;
        state.serialize_field("lifestyle", &self.lifestyle)?;
        state.serialize_field("perks", &self.perks)?;
        state.end()
    }
}
