use serde::Serialize;

use super::{
    super::{
        display::Cullable,
        game_data::{Localizable, Localize},
        parser::{
            GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError, SaveFileValue,
        },
        types::{Wrapper, WrapperMut},
    },
    Character, FromGameObject, GameId, GameObjectDerived, Shared,
};

/// A struct representing a lineage node in the game
#[derive(Serialize)]
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

impl FromGameObject for LineageNode {
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let id = base.get_game_id("character")?;
        let char = game_state.get_character(&id);
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
            character: Some(char),
            date: Some(base.get_string("date")?),
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
            id: id,
        })
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

impl Localizable for LineageNode {
    fn localize<L: Localize>(&mut self, localization: &mut L) {
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
