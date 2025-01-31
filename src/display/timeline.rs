use std::path::{Path, PathBuf};

use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        game_data::GameData,
        jinja_env::TIMELINE_TEMPLATE_NAME,
        parser::{GameId, GameState, GameString},
        structures::{Character, Culture, DerivedRef, Faith, GameObjectDerived, Title},
        types::{Shared, Wrapper, WrapperMut},
    },
    graph::{create_timeline_graph, Grapher},
    renderer::{Cullable, Renderable},
    RenderableType,
};

/// An enum representing the difference in faith or culture between two realms, really just a wrapper around DerivedRef
pub enum RealmDifference {
    Faith(Shared<Faith>),
    Culture(Shared<Culture>),
}

impl Serialize for RealmDifference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RealmDifference::Faith(f) => f.serialize(serializer),
            RealmDifference::Culture(c) => c.serialize(serializer),
        }
    }
}

impl GameObjectDerived for RealmDifference {
    fn get_id(&self) -> GameId {
        0
    }

    fn get_name(&self) -> GameString {
        match self {
            RealmDifference::Faith(f) => f.get_internal().get_name(),
            RealmDifference::Culture(c) => c.get_internal().get_name(),
        }
    }
}

impl Cullable for RealmDifference {
    fn get_depth(&self) -> usize {
        match self {
            RealmDifference::Faith(f) => f.get_internal().get_depth(),
            RealmDifference::Culture(c) => c.get_internal().get_depth(),
        }
    }

    fn is_ok(&self) -> bool {
        match self {
            RealmDifference::Faith(f) => f.get_internal().is_ok(),
            RealmDifference::Culture(c) => c.get_internal().is_ok(),
        }
    }

    fn set_depth(&mut self, depth: usize) {
        match self {
            RealmDifference::Faith(f) => f.get_internal_mut().set_depth(depth),
            RealmDifference::Culture(c) => c.get_internal_mut().set_depth(depth),
        }
    }
}

/// A struct representing the timeline of the game
pub struct Timeline {
    lifespans: Vec<(Shared<Title>, Vec<(i16, i16)>)>,
    latest_event: i16,
    events: Vec<(
        i16,
        Shared<Character>,
        Shared<Title>,
        GameString,
        RealmDifference,
    )>, // (year, character, title, event_type<conquered, usurped, etc.
}

impl Timeline {
    /// Creates a new timeline from the game state
    pub fn new(
        lifespans: Vec<(Shared<Title>, Vec<(i16, i16)>)>,
        latest_event: i16,
        events: Vec<(
            i16,
            Shared<Character>,
            Shared<Title>,
            GameString,
            RealmDifference,
        )>,
    ) -> Self {
        Self {
            lifespans,
            latest_event,
            events,
        }
    }
}

enum RealmDifferenceRef {
    Faith(DerivedRef<Faith>),
    Culture(DerivedRef<Culture>),
}

impl Serialize for RealmDifferenceRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RealmDifferenceRef::Faith(f) => f.serialize(serializer),
            RealmDifferenceRef::Culture(c) => c.serialize(serializer),
        }
    }
}

impl Serialize for Timeline {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Timeline", 3)?;
        let ref_lifespans: Vec<(DerivedRef<Title>, Vec<(i16, i16)>)> = self
            .lifespans
            .iter()
            .map(|(t, v)| (DerivedRef::from(t.clone()), v.clone()))
            .collect();
        state.serialize_field("lifespans", &ref_lifespans)?;
        state.serialize_field("latest_event", &self.latest_event)?;
        let mut ref_events = Vec::new();
        for events in &self.events {
            let (year, char, title, event_type, difference) = events;
            let ref_char = DerivedRef::from(char.clone());
            let ref_title = DerivedRef::from(title.clone());
            let ref_diff = match difference {
                RealmDifference::Faith(f) => RealmDifferenceRef::Faith(DerivedRef::from(f.clone())),
                RealmDifference::Culture(c) => {
                    RealmDifferenceRef::Culture(DerivedRef::from(c.clone()))
                }
            };
            ref_events.push((
                year.clone(),
                ref_char,
                ref_title,
                event_type.clone(),
                ref_diff,
            ));
        }
        state.serialize_field("events", &ref_events)?;
        state.end()
    }
}

impl GameObjectDerived for Timeline {
    fn get_id(&self) -> GameId {
        0
    }

    fn get_name(&self) -> GameString {
        GameString::wrap("Timeline".to_string())
    }
}

impl Cullable for Timeline {
    fn get_depth(&self) -> usize {
        0
    }

    fn is_ok(&self) -> bool {
        true
    }

    fn set_depth(&mut self, depth: usize) {
        for (title, _) in self.lifespans.iter_mut() {
            title.get_internal_mut().set_depth(depth);
        }
        for (_, char, title, _, difference) in self.events.iter_mut() {
            char.get_internal_mut().set_depth(depth);
            title.get_internal_mut().set_depth(depth);
            match difference {
                RealmDifference::Faith(f) => f.get_internal_mut().set_depth(depth),
                RealmDifference::Culture(c) => c.get_internal_mut().set_depth(depth),
            }
        }
    }
}

impl Renderable for Timeline {
    fn get_subdir() -> &'static str {
        "."
    }

    fn get_path(&self, path: &Path) -> PathBuf {
        path.join("timeline.html")
    }

    fn get_template() -> &'static str {
        TIMELINE_TEMPLATE_NAME
    }

    fn render(&self, path: &Path, _: &GameState, grapher: Option<&Grapher>, _: &GameData) {
        if grapher.is_some() {
            create_timeline_graph(
                &self.lifespans,
                self.latest_event,
                path.join("timeline.svg"),
            );
        }
    }

    fn append_ref(&self, stack: &mut Vec<RenderableType>) {
        for (title, _) in &self.lifespans {
            stack.push(RenderableType::Title(title.clone()));
        }

        for (_, char, _, _, difference) in &self.events {
            stack.push(RenderableType::Character(char.clone()));
            match difference {
                RealmDifference::Faith(f) => stack.push(RenderableType::Faith(f.clone())),
                RealmDifference::Culture(c) => stack.push(RenderableType::Culture(c.clone())),
            }
        }
    }
}
