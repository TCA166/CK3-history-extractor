use std::path::{Path, PathBuf};

use super::{
    super::{
        game_data::GameData,
        save_file::{
            parser::types::GameString,
            structures::{Character, Culture, EntityRef, Faith, GameObjectDerived, GameRef, Title},
            GameState,
        },
    },
    graph::{create_timeline_graph, Grapher},
    renderer::{GetPath, Renderable},
};

/// An enum representing the difference in faith or culture between two realms, really just a wrapper around DerivedRef
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(untagged))]
pub enum RealmDifference {
    Faith(GameRef<Faith>),
    Culture(GameRef<Culture>),
}

/// A struct representing the timeline of the game
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Timeline {
    lifespans: Vec<(GameRef<Title>, Vec<(i16, i16)>)>,
    latest_event: i16,
    events: Vec<(
        i16,
        GameRef<Character>,
        GameRef<Title>,
        GameString,
        RealmDifference,
    )>, // (year, character, title, event_type<conquered, usurped, etc.
}

impl Timeline {
    /// Creates a new timeline from the game state
    pub fn new(
        lifespans: Vec<(GameRef<Title>, Vec<(i16, i16)>)>,
        latest_event: i16,
        events: Vec<(
            i16,
            GameRef<Character>,
            GameRef<Title>,
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

impl GameObjectDerived for Timeline {
    fn get_name(&self) -> GameString {
        GameString::from("Timeline")
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
        for (title, _) in &self.lifespans {
            collection.extend([E::from(title.clone().into())]);
        }

        for (_, char, _, _, difference) in &self.events {
            collection.extend([E::from(char.clone().into())]);
            match difference {
                RealmDifference::Faith(f) => collection.extend([E::from(f.clone().into())]),
                RealmDifference::Culture(c) => collection.extend([E::from(c.clone().into())]),
            }
        }
    }
}

impl GetPath for Timeline {
    fn get_path(&self, path: &Path) -> PathBuf {
        path.join("timeline.html")
    }
}

impl Renderable for Timeline {
    const TEMPLATE_NAME: &'static str = "timelineTemplate";

    fn render(&self, path: &Path, _: &GameState, grapher: Option<&Grapher>, _: &GameData) {
        if grapher.is_some() {
            create_timeline_graph(
                &self.lifespans,
                self.latest_event,
                path.join("timeline.svg"),
            );
        }
    }
}
