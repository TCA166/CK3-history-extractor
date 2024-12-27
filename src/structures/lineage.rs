use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        display::{Cullable, Localizable, Localizer},
        parser::{GameObjectMap, GameState, GameString, SaveFileValue},
        types::{Wrapper, WrapperMut},
    },
    Character, FromGameObject, GameId, GameObjectDerived, Shared,
};

/// A struct representing a lineage node in the game
pub struct LineageNode {
    character: Option<Shared<Character>>,
    date: Option<GameString>,
    score: i32,
    prestige: i32,
    piety: i32,
    dread: f32,
    lifestyle: Option<GameString>,
    perks: Vec<GameString>, //in older CK3 version this was a list, guess it no longer is
    id: GameId,
}

impl LineageNode {
    /// Gets the character associated with the lineage node
    pub fn get_character(&self) -> Shared<Character> {
        self.character.as_ref().unwrap().clone()
    }
}

///Gets the perk of the lineage node
fn get_perks(perks: &mut Vec<GameString>, base: &GameObjectMap) {
    if let Some(perks_node) = base.get("perk") {
        match perks_node {
            SaveFileValue::Object(o) => {
                for perk in o.as_array() {
                    perks.push(perk.as_string())
                }
            }
            SaveFileValue::String(o) => {
                perks.push(o.clone());
            }
            _ => {
                unreachable!()
            }
        }
    }
}

///Gets the dread of the lineage node
fn get_dread(base: &GameObjectMap) -> f32 {
    if let Some(dread_node) = base.get("dread") {
        dread_node.as_string().parse::<f32>().unwrap()
    } else {
        0.0
    }
}

///Gets the score of the lineage node
fn get_score(base: &GameObjectMap) -> i32 {
    if let Some(score_node) = base.get("score") {
        score_node.as_string().parse::<i32>().unwrap()
    } else {
        0
    }
}

///Gets the prestige of the lineage node
fn get_prestige(base: &GameObjectMap) -> i32 {
    if let Some(prestige_node) = base.get("prestige") {
        prestige_node.as_string().parse::<i32>().unwrap()
    } else {
        0
    }
}

///Gets the piety of the lineage node
fn get_piety(base: &GameObjectMap) -> i32 {
    if let Some(piety_node) = base.get("piety") {
        piety_node.as_string().parse::<i32>().unwrap()
    } else {
        0
    }
}

///Gets the lifestyle of the lineage node
fn get_lifestyle(base: &GameObjectMap) -> Option<GameString> {
    if let Some(lifestyle_node) = base.get("lifestyle") {
        Some(lifestyle_node.as_string())
    } else {
        None
    }
}

impl FromGameObject for LineageNode {
    fn from_game_object(base: &GameObjectMap, game_state: &mut GameState) -> Self {
        let id = base.get("character").unwrap().as_id();
        let char = game_state.get_character(&id);
        let mut perks = Vec::new();
        get_perks(&mut perks, base);
        LineageNode {
            character: Some(char),
            date: Some(base.get("date").unwrap().as_string()),
            score: get_score(&base),
            prestige: get_prestige(&base),
            piety: get_piety(&base),
            dread: get_dread(&base),
            lifestyle: get_lifestyle(&base),
            perks: perks,
            id: id,
        }
    }
}

impl GameObjectDerived for LineageNode {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        self.character.as_ref().unwrap().get_internal().get_name()
    }
}

impl Serialize for LineageNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("LineageNode", 9)?;
        state.serialize_field("character", &self.character)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("score", &self.score)?;
        state.serialize_field("prestige", &self.prestige)?;
        state.serialize_field("piety", &self.piety)?;
        state.serialize_field("dread", &self.dread)?;
        state.serialize_field("lifestyle", &self.lifestyle)?;
        state.serialize_field("perks", &self.perks)?;
        state.serialize_field("id", &self.id)?;
        state.end()
    }
}

impl Localizable for LineageNode {
    fn localize(&mut self, localization: &mut Localizer) {
        if let Some(lifestyle) = &self.lifestyle {
            self.lifestyle = Some(localization.localize(&lifestyle.as_str()));
        }
        for perk in self.perks.iter_mut() {
            *perk = localization.localize(perk.as_str());
        }
    }
}

impl Cullable for LineageNode {
    fn get_depth(&self) -> usize {
        self.character.as_ref().unwrap().get_internal().get_depth()
    }

    fn set_depth(&mut self, depth: usize) {
        self.character
            .as_ref()
            .unwrap()
            .get_internal_mut()
            .set_depth(depth);
    }
}
