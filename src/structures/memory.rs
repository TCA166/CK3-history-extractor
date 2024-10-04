use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        display::{Cullable, Localizer, RenderableType},
        parser::{GameId, GameObjectMap, GameState, GameString},
        types::WrapperMut,
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
    localized: bool,
}

/// Gets the participants of the memory and appends them to the participants vector
fn get_participants(
    participants: &mut Vec<(String, Shared<Character>)>,
    base: &GameObjectMap,
    game_state: &mut GameState,
) {
    let participants_node = base.get("participants");
    if participants_node.is_some() {
        for part in participants_node.unwrap().as_object().as_map() {
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
            localized: false,
        }
    }

    fn init(&mut self, base: &GameObjectMap, game_state: &mut GameState) {
        self.date = Some(base.get("creation_date").unwrap().as_string());
        let tp = base.get("type");
        if tp.is_some() {
            self.r#type = Some(tp.unwrap().as_string());
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

impl Cullable for Memory {
    fn set_depth(&mut self, depth: usize, localization: &Localizer) {
        if depth <= self.depth && depth != 0 {
            return;
        }
        if !self.localized {
            self.r#type = Some(localization.localize(&self.r#type.as_ref().unwrap()));
        }
        if depth == 0 {
            return;
        }
        self.depth = depth;
        for part in self.participants.iter_mut() {
            if !self.localized {
                part.0 = localization.localize(&part.0).to_string();
            }
            let o = part.1.try_get_internal_mut();
            if o.is_ok() {
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
        self.localized = true;
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
