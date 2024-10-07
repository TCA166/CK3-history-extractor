use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        display::{Cullable, Localizable, Localizer, RenderableType},
        parser::{GameId, GameObjectMap, GameState, GameString},
    },
    Character, DerivedRef, DummyInit, GameObjectDerived, Shared,
};

/// A struct representing a memory in the game
pub struct Memory {
    id: GameId,
    date: Option<GameString>,
    r#type: Option<GameString>,
    participants: Vec<(String, Shared<Character>)>,
    depth: usize,
}

/// Gets the participants of the memory and appends them to the participants vector
fn get_participants(
    participants: &mut Vec<(String, Shared<Character>)>,
    base: &GameObjectMap,
    game_state: &mut GameState,
) {
    if let Some(participants_node) = base.get("participants") {
        for part in participants_node.as_object().as_map() {
            participants.push((
                part.0.clone(),
                game_state.get_character(&part.1.as_id()).clone(),
            ));
        }
    }
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

    fn init(&mut self, base: &GameObjectMap, game_state: &mut GameState) {
        self.date = Some(base.get("creation_date").unwrap().as_string());
        if let Some(tp) = base.get("type") {
            self.r#type = Some(tp.as_string());
        }
        get_participants(&mut self.participants, &base, game_state);
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

impl Serialize for Memory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Memory", 3)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("type", &self.r#type)?;
        // serialize the participants as an array of tuples
        let mut participants: Vec<(String, DerivedRef<Character>)> = Vec::new();
        for part in self.participants.iter() {
            participants.push((part.0.clone(), DerivedRef::from_derived(part.1.clone())));
        }
        state.serialize_field("participants", &participants)?;
        state.end()
    }
}

impl Localizable for Memory {
    fn localize(&mut self, localization: &Localizer) {
        self.r#type = Some(localization.localize(&self.r#type.as_ref().unwrap()));
        for part in self.participants.iter_mut() {
            part.0 = localization.localize(&part.0).to_string();
        }
    }
}

impl Cullable for Memory {
    fn set_depth(&mut self, depth: usize) {
        if depth <= self.depth {
            return;
        }
        self.depth = depth;
        for part in self.participants.iter_mut() {
            if let Ok(mut part) = part.1.try_borrow_mut() {
                part.set_depth(depth - 1);
            }
        }
    }

    fn get_depth(&self) -> usize {
        self.depth
    }
}

impl Memory {
    pub fn render_participants(&self, stack: &mut Vec<RenderableType>) {
        for part in self.participants.iter() {
            stack.push(RenderableType::Character(part.1.clone()));
        }
    }
}
