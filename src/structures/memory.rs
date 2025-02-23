use jomini::common::Date;
use serde::Serialize;

use super::{
    super::{
        game_data::{Localizable, LocalizationError, Localize},
        parser::{GameId, GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError},
    },
    Character, GameObjectDerived, Shared, Wrapper,
};

/// A struct representing a memory in the game
#[derive(Serialize, Debug)]
pub struct Memory {
    id: GameId,
    date: Option<Date>,
    r#type: Option<GameString>,
    participants: Vec<(String, Shared<Character>)>,
}

impl DummyInit for Memory {
    fn dummy(id: GameId) -> Self {
        Memory {
            date: None,
            r#type: None,
            participants: Vec::new(),
            id: id,
        }
    }

    fn init(
        &mut self,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<(), ParsingError> {
        self.date = Some(base.get_date("creation_date")?);
        if let Some(tp) = base.get("type") {
            self.r#type = Some(tp.as_string()?);
        }
        if let Some(participants_node) = base.get("participants") {
            for part in participants_node.as_object()?.as_map()? {
                self.participants.push((
                    part.0.clone(),
                    game_state.get_character(&part.1.as_id()?).clone(),
                ));
            }
        }
        Ok(())
    }
}

impl GameObjectDerived for Memory {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> Option<GameString> {
        self.r#type.as_ref().map(|s| s.clone())
    }

    fn get_references<E: From<GameObjectDerivedType>, C: Extend<E>>(&self, collection: &mut C) {
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
        self.r#type = Some(localization.localize(&self.r#type.as_ref().unwrap())?);
        // there are no worthy localization keys for the relation names, so we don't localize them
        /*
        for part in self.participants.iter_mut() {
            part.0 = localization.localize(&part.0)?.to_string();
        }
        */
        Ok(())
    }
}

impl Serialize for Shared<Memory> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.get_internal().serialize(serializer)
    }
}
