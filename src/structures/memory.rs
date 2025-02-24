use jomini::common::Date;
use serde::Serialize;

use super::{
    super::{
        game_data::{Localizable, LocalizationError, Localize},
        parser::{GameId, GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError},
        types::Wrapper,
    },
    Character, EntityRef, FromGameObject, GameObjectDerived, GameRef,
};

/// A struct representing a memory in the game
#[derive(Serialize, Debug)]
pub struct Memory {
    date: Date,
    r#type: GameString,
    participants: Vec<(String, GameRef<Character>)>,
}

impl FromGameObject for Memory {
    fn from_game_object(
        _id: GameId,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let mut val = Self {
            date: base.get_date("creation_date")?,
            r#type: base.get_string("type")?,
            participants: Vec::new(),
        };
        if let Some(participants_node) = base.get("participants") {
            for part in participants_node.as_object()?.as_map()? {
                val.participants.push((
                    part.0.clone(),
                    game_state.get_character(&part.1.as_id()?).clone(),
                ));
            }
        }
        Ok(val)
    }
}

impl GameObjectDerived for Memory {
    fn get_name(&self) -> GameString {
        self.r#type.clone()
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
        for part in self.participants.iter() {
            collection.extend([E::from(part.1.clone().into())]);
        }
    }
}

impl Localizable for Memory {
    fn localize<L: Localize<GameString>>(
        &mut self,
        localization: &mut L,
    ) -> Result<(), LocalizationError> {
        self.r#type = localization.localize(&self.r#type)?;
        // there are no worthy localization keys for the relation names, so we don't localize them
        /*
        for part in self.participants.iter_mut() {
            part.0 = localization.localize(&part.0)?.to_string();
        }
        */
        Ok(())
    }
}

impl Serialize for GameRef<Memory> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get_internal().serialize(serializer)
    }
}
