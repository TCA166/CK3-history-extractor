use serde::{ser::SerializeStruct, Serialize};

use super::super::{
    display::{Cullable, Localizer},
    game_object::{GameObject, GameString, SaveFileValue},
    game_state::GameState,
    types::{Wrapper, WrapperMut},
};

use super::{Character, FromGameObject, GameId, GameObjectDerived, Shared};

/// A struct representing a lineage node in the game
pub struct LineageNode {
    character: Option<Shared<Character>>,
    date: Option<GameString>,
    score: i32,
    prestige: i32,
    piety: i32,
    dread: f32,
    lifestyle: Option<GameString>,
    perks: Vec<GameString>, //in older version this was a list, guess it no longer is
    id: GameId,
}

impl LineageNode {
    /// Gets the character associated with the lineage node
    pub fn get_character(&self) -> Shared<Character> {
        self.character.as_ref().unwrap().clone()
    }
}

///Gets the perk of the lineage node
fn get_perks(perks: &mut Vec<GameString>, base: &GameObject) {
    let perks_node = base.get("perk");
    if perks_node.is_some() {
        let node = perks_node.unwrap();
        match node {
            SaveFileValue::Object(o) => {
                for perk in o.get_array_iter() {
                    perks.push(perk.as_string())
                }
            }
            SaveFileValue::String(o) => {
                perks.push(o.clone());
            }
        }
    }
}

///Gets the dread of the lineage node
fn get_dread(base: &GameObject) -> f32 {
    let dread;
    let dread_node = base.get("dread");
    if dread_node.is_some() {
        dread = dread_node.unwrap().as_string().parse::<f32>().unwrap();
    } else {
        dread = 0.0;
    }
    return dread;
}

///Gets the score of the lineage node
fn get_score(base: &GameObject) -> i32 {
    let score;
    let score_node = base.get("score");
    if score_node.is_some() {
        score = score_node.unwrap().as_string().parse::<i32>().unwrap();
    } else {
        score = 0;
    }
    score
}

///Gets the prestige of the lineage node
fn get_prestige(base: &GameObject) -> i32 {
    let prestige;
    let prestige_node = base.get("prestige");
    if prestige_node.is_some() {
        prestige = prestige_node.unwrap().as_string().parse::<i32>().unwrap();
    } else {
        prestige = 0;
    }
    prestige
}

///Gets the piety of the lineage node
fn get_piety(base: &GameObject) -> i32 {
    let piety;
    let piety_node = base.get("piety");
    if piety_node.is_some() {
        piety = piety_node.unwrap().as_string().parse::<i32>().unwrap();
    } else {
        piety = 0;
    }
    piety
}

///Gets the lifestyle of the lineage node
fn get_lifestyle(base: &GameObject) -> Option<GameString> {
    let lifestyle_node = base.get("lifestyle");
    if lifestyle_node.is_some() {
        Some(lifestyle_node.unwrap().as_string())
    } else {
        None
    }
}

impl FromGameObject for LineageNode {
    fn from_game_object(base: &GameObject, game_state: &mut GameState) -> Self {
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

impl Cullable for LineageNode {
    fn get_depth(&self) -> usize {
        self.character.as_ref().unwrap().get_internal().get_depth()
    }

    fn set_depth(&mut self, depth: usize, localization: &Localizer) {
        if self.lifestyle.is_some() {
            self.lifestyle = Some(localization.localize(self.lifestyle.as_ref().unwrap().as_str()));
        }
        for perk in self.perks.iter_mut() {
            *perk = localization.localize(perk.as_str());
        }
        self.character
            .as_ref()
            .unwrap()
            .get_internal_mut()
            .set_depth(depth, localization);
    }
}
