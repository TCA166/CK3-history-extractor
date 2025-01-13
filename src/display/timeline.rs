use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        game_data::GameMap,
        jinja_env::TIMELINE_TEMPLATE_NAME,
        parser::{GameId, GameState, GameString},
        structures::{Character, Culture, DerivedRef, Faith, GameObjectDerived, Title},
        types::{Shared, Wrapper, WrapperMut},
    },
    graph::Grapher,
    renderer::{Cullable, Renderable},
    RenderableType,
};

//const CREATED_STR:&str = "Created";
const DESTROYED_STR: &str = "destroyed";
const USURPED_STR: &str = "usurped";
const CONQUERED_START_STR: &str = "conq"; //this should match both 'conquered' and 'conquest holy war'

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
    lifespans: Vec<(Shared<Title>, Vec<(u32, u32)>)>,
    latest_event: u32,
    events: Vec<(
        u32,
        Shared<Character>,
        Shared<Title>,
        GameString,
        RealmDifference,
    )>, // (year, character, title, event_type<conquered, usurped, etc.
}

impl Timeline {
    /// Creates a new timeline from the game state
    pub fn new(game_state: &GameState) -> Self {
        let mut lifespans = Vec::new();
        let mut latest_event = 0;
        let mut event_checkout = Vec::new();
        for (_, title) in game_state.get_title_iter() {
            //first we handle the empires and collect titles that might be relevant for events
            let t = title.get_internal();
            let hist = t.get_history_iter();
            if hist.len() == 0 {
                continue;
            }
            if let Some(k) = t.get_key() {
                //if the key is there
                let kingdom = k.as_ref().starts_with("k_");
                if kingdom {
                    event_checkout.push(title.clone());
                    //event_checkout.push(title.get_internal().get_capital().unwrap().clone());
                    continue;
                }
                let empire = k.as_ref().starts_with("e_");
                if !empire {
                    continue;
                }
                event_checkout.push(title.clone());
                event_checkout.push(title.get_internal().get_capital().unwrap().clone());
                let mut item = (title.clone(), Vec::new());
                let mut empty = true;
                let mut start = 0;
                for entry in hist {
                    let yr = entry.0.split_once('.').unwrap().0.parse().unwrap();
                    if yr > latest_event {
                        latest_event = yr;
                    }
                    let event = entry.2.as_str();
                    if event == DESTROYED_STR {
                        //if it was destroyed we mark the end of the lifespan
                        item.1.push((start, yr));
                        empty = true;
                    } else if empty {
                        //else if we are not in a lifespan we start a new one
                        start = yr;
                        empty = false;
                    }
                }
                if empire {
                    if !empty {
                        item.1.push((start, 0));
                    }
                    //println!("{} {:?}", title.get_internal().get_key().unwrap(), item.1);
                    lifespans.push(item);
                }
            }
        }
        let mut events: Vec<(
            u32,
            Shared<Character>,
            Shared<Title>,
            GameString,
            RealmDifference,
        )> = Vec::new();
        for title in event_checkout {
            let tit = title.get_internal();
            //find the first event that has a character attached
            let mut hist = tit.get_history_iter().skip_while(|a| a.1.is_none());
            let next = hist.next();
            if next.is_none() {
                continue;
            }
            let first_char = next.unwrap().1.as_ref().unwrap().get_internal();
            let mut faith = first_char.get_faith().unwrap().get_internal().get_id();
            let mut culture = first_char.get_culture().unwrap().get_internal().get_id();
            for entry in hist {
                let char = entry.1.as_ref();
                if char.is_none() {
                    continue;
                }
                let char = char.unwrap();
                let event = entry.2.as_str();
                let ch = char.get_internal();
                let char_faith = ch.get_faith();
                let ch_faith = char_faith.as_ref().unwrap().get_internal();
                let char_culture = ch.get_culture();
                let ch_culture = char_culture.as_ref().unwrap().get_internal();
                if event == USURPED_STR || event.starts_with(CONQUERED_START_STR) {
                    let year: u32 = entry.0.split_once('.').unwrap().0.parse().unwrap();
                    if ch_faith.get_id() != faith {
                        events.push((
                            year,
                            char.clone(),
                            title.clone(),
                            GameString::wrap("faith".to_owned()),
                            RealmDifference::Faith(char_faith.as_ref().unwrap().clone()),
                        ));
                        faith = ch_faith.get_id();
                    } else if ch_culture.get_id() != culture {
                        events.push((
                            year,
                            char.clone(),
                            title.clone(),
                            GameString::wrap("people".to_owned()),
                            RealmDifference::Culture(char_culture.as_ref().unwrap().clone()),
                        ));
                        culture = ch_culture.get_id();
                    }
                } else {
                    if ch_faith.get_id() != faith {
                        faith = ch_faith.get_id();
                    }
                    if ch_culture.get_id() != culture {
                        culture = ch_culture.get_id();
                    }
                }
            }
        }
        events.sort_by(|a, b| a.0.cmp(&b.0));
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
        let ref_lifespans: Vec<(DerivedRef<Title>, Vec<(u32, u32)>)> = self
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

    fn get_path(&self, path: &str) -> String {
        format!("{}/timeline.html", path)
    }

    fn get_template() -> &'static str {
        TIMELINE_TEMPLATE_NAME
    }

    fn render(&self, path: &str, _: &GameState, grapher: Option<&Grapher>, _: Option<&GameMap>) {
        if grapher.is_some() {
            let path = format!("{}/timeline.svg", path);
            Grapher::create_timeline_graph(&self.lifespans, self.latest_event, &path)
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
