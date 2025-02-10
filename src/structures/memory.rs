use jomini::common::Date;
use serde::{ser::SerializeSeq, Serialize};

use super::{
    super::{
        display::{Cullable, RenderableType},
        game_data::{Localizable, LocalizationError, Localize},
        parser::{GameId, GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError},
        types::WrapperMut,
    },
    Character, DerivedRef, DummyInit, GameObjectDerived, Shared,
};

fn serialize_participants<S: serde::Serializer>(
    val: &Vec<(String, Shared<Character>)>,
    s: S,
) -> Result<S::Ok, S::Error> {
    let mut seq = s.serialize_seq(Some(val.len()))?;
    for (name, character) in val {
        let character = DerivedRef::from(character.clone());
        seq.serialize_element(&(name, character))?;
    }
    seq.end()
}

/// A struct representing a memory in the game
#[derive(Serialize)]
pub struct Memory {
    id: GameId,
    date: Option<Date>,
    r#type: Option<GameString>,
    #[serde(serialize_with = "serialize_participants")]
    participants: Vec<(String, Shared<Character>)>,
    depth: usize,
}

impl DummyInit for Memory {
    fn dummy(id: GameId) -> Self {
        Memory {
            date: None,
            r#type: None,
            participants: Vec::new(),
            id: id,
            depth: 0,
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

    fn get_name(&self) -> GameString {
        self.r#type.as_ref().unwrap().clone()
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

impl Cullable for Memory {
    fn set_depth(&mut self, depth: usize) {
        if depth <= self.depth {
            return;
        }
        self.depth = depth;
        for part in self.participants.iter_mut() {
            if let Ok(mut part) = part.1.try_get_internal_mut() {
                part.set_depth(depth - 1);
            }
        }
    }

    fn get_depth(&self) -> usize {
        self.depth
    }
}

impl Memory {
    pub fn add_participants(&self, stack: &mut Vec<RenderableType>) {
        for part in self.participants.iter() {
            stack.push(RenderableType::Character(part.1.clone()));
        }
    }
}
