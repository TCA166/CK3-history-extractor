use std::cmp::Ordering;
use std::slice::Iter;

use minijinja::context;

use serde::{ser::SerializeStruct, Serialize};

use super::super::{
    display::{Cullable, Localizer, Renderable, RenderableType, Renderer},
    game_object::{GameObject, GameString, SaveFileValue},
    game_state::GameState,
    types::{Wrapper, WrapperMut},
};
use super::{serialize_array, Character, DerivedRef, DummyInit, GameId, GameObjectDerived, Shared};

/// A struct representing a title in the game
pub struct Title {
    id: GameId,
    key: Option<GameString>,
    name: Option<GameString>,
    de_jure: Option<Shared<Title>>,
    de_facto: Option<Shared<Title>>,
    de_jure_vassals: Vec<Shared<Title>>,
    de_facto_vassals: Vec<Shared<Title>>,
    history: Vec<(GameString, Option<Shared<Character>>, GameString)>,
    claims: Vec<Shared<Character>>,
    depth: usize,
    localized: bool,
    name_localized: bool,
    capital: Option<Shared<Title>>,
    color: [u8; 3],
}

/// Compares two date strings in the format "YYYY.MM.DD" and returns the ordering
fn date_string_cmp(a: &str, b: &str) -> Ordering {
    let a_split: Vec<&str> = a.split('.').collect();
    let b_split: Vec<&str> = b.split('.').collect();
    let a_year = a_split[0].parse::<u16>().unwrap();
    let b_year = b_split[0].parse::<u16>().unwrap();
    if a_year < b_year {
        return Ordering::Less;
    } else if a_year > b_year {
        return Ordering::Greater;
    }
    let a_month = a_split[1].parse::<u8>().unwrap();
    let b_month = b_split[1].parse::<u8>().unwrap();
    if a_month < b_month {
        return Ordering::Less;
    } else if a_month > b_month {
        return Ordering::Greater;
    }
    let a_day = a_split[2].parse::<u8>().unwrap();
    let b_day = b_split[2].parse::<u8>().unwrap();
    if a_day < b_day {
        return Ordering::Less;
    } else if a_day > b_day {
        return Ordering::Greater;
    }
    Ordering::Equal
}

///Gets the history of the title and returns a hashmap with the history entries
fn get_history(
    base: &GameObject,
    game_state: &mut GameState,
) -> Vec<(GameString, Option<Shared<Character>>, GameString)> {
    let mut history: Vec<(GameString, Option<Shared<Character>>, GameString)> = Vec::new();
    let hist = base.get("history");
    if hist.is_some() {
        let hist_obj = hist.unwrap().as_object().unwrap();
        for h in hist_obj.get_keys() {
            let val = hist_obj.get(&h);
            let character;
            let action: GameString;
            match val {
                Some(&SaveFileValue::Object(ref o)) => {
                    if o.is_array() {
                        for entry in o.get_array_iter() {
                            let loc_action;
                            let loc_character;
                            match entry {
                                SaveFileValue::Object(ref o) => {
                                    loc_action = o.get("type").unwrap().as_string();
                                    let holder = o.get("holder");
                                    match holder {
                                        Some(h) => {
                                            loc_character =
                                                Some(game_state.get_character(&h.as_id()).clone());
                                        }
                                        None => {
                                            loc_character = None;
                                        }
                                    }
                                }
                                SaveFileValue::String(ref o) => {
                                    loc_action = GameString::wrap("Inherited".to_owned());
                                    loc_character = Some(
                                        game_state
                                            .get_character(&o.parse::<GameId>().unwrap())
                                            .clone(),
                                    );
                                }
                            }
                            history.push((
                                GameString::wrap(h.to_string()),
                                loc_character,
                                loc_action,
                            ))
                        }
                        continue; //if it's an array we handled all the adding already in the loop above
                    } else {
                        action = o.get("type").unwrap().as_string();
                        let holder = o.get("holder");
                        match holder {
                            Some(h) => {
                                character = Some(game_state.get_character(&h.as_id()).clone());
                            }
                            None => {
                                character = None;
                            }
                        }
                    }
                }
                Some(&SaveFileValue::String(ref o)) => {
                    action = GameString::wrap("Inherited".to_owned());
                    character = Some(
                        game_state
                            .get_character(&o.parse::<GameId>().unwrap())
                            .clone(),
                    );
                }
                _ => {
                    panic!("Invalid history entry")
                }
            }
            history.push((GameString::wrap(h.to_string()), character, action));
        }
    }
    //sort history by the first element of the tuple (the date) in descending order
    history.sort_by(|a, b| date_string_cmp(a.0.as_str(), b.0.as_str()));
    history
}

impl Title {
    /// Adds a de jure vassal to the title
    pub fn add_jure_vassal(&mut self, vassal: Shared<Title>) {
        self.de_jure_vassals.push(vassal);
    }

    /// Adds a de facto vassal to the title
    pub fn add_facto_vassal(&mut self, vassal: Shared<Title>) {
        self.de_facto_vassals.push(vassal);
    }

    /// Recursively gets all the de facto barony keys of the title
    pub fn get_barony_keys(&self) -> Vec<GameString> {
        let mut provinces = Vec::new();
        if self.key.as_ref().unwrap().starts_with("b_") {
            provinces.push(self.key.clone().unwrap());
        }
        for v in &self.de_facto_vassals {
            provinces.append(&mut v.get_internal().get_barony_keys());
        }
        provinces
    }

    pub fn get_key(&self) -> Option<GameString> {
        self.key.clone()
    }

    pub fn get_history_iter(&self) -> Iter<(GameString, Option<Shared<Character>>, GameString)> {
        self.history.iter()
    }

    pub fn get_capital(&self) -> Option<Shared<Title>> {
        self.capital.clone()
    }
}

impl DummyInit for Title {
    fn dummy(id: GameId) -> Self {
        Title {
            key: None,
            name: None,
            de_jure: None,
            de_facto: None,
            de_jure_vassals: Vec::new(),
            de_facto_vassals: Vec::new(),
            history: Vec::new(),
            claims: Vec::new(),
            id: id,
            depth: 0,
            localized: false,
            name_localized: false,
            color: [70, 255, 70],
            capital: None,
        }
    }

    fn init(&mut self, base: &GameObject, game_state: &mut GameState) {
        if base.get_array_iter().len() > 3 {
            let color = base
                .get_array_iter()
                .map(|x| x.as_string().parse().unwrap())
                .collect::<Vec<u8>>();
            self.color = [color[0], color[1], color[2]];
        }
        self.key = Some(base.get_string_ref("key"));
        let de_jure_id = base.get("de_jure_liege");
        if de_jure_id.is_some() {
            let o = game_state.get_title(&de_jure_id.unwrap().as_id()).clone();
            self.de_jure = Some(o.clone());
            o.get_internal_mut()
                .add_jure_vassal(game_state.get_title(&self.id).clone());
        }
        let de_facto_id = base.get("de_facto_liege");
        if de_facto_id.is_some() {
            let o = game_state.get_title(&de_facto_id.unwrap().as_id()).clone();
            self.de_facto = Some(o.clone());
            o.get_internal_mut()
                .add_facto_vassal(game_state.get_title(&self.id).clone());
        }
        let claim_node = base.get("claim");
        if claim_node.is_some() {
            let c = claim_node.unwrap();
            if let Some(claim) = c.as_object() {
                for claim in claim.get_array_iter() {
                    self.claims
                        .push(game_state.get_character(&claim.as_id()).clone());
                }
            } else {
                self.claims
                    .push(game_state.get_character(&c.as_id()).clone());
            }
        }
        let capital = base.get("capital");
        if capital.is_some() {
            self.capital = Some(game_state.get_title(&capital.unwrap().as_id()).clone());
        }
        self.name = Some(base.get("name").unwrap().as_string().clone());
        let history = get_history(base, game_state);
        self.history = history;
    }
}

impl GameObjectDerived for Title {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        if self.name.is_none() {
            return GameString::wrap("Unnamed".to_owned());
        }
        self.name.as_ref().unwrap().clone()
    }
}

impl Serialize for Title {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Title", 10)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        if self.de_jure.is_some() {
            let de_jure = DerivedRef::from_derived(self.de_jure.as_ref().unwrap().clone());
            state.serialize_field("de_jure", &de_jure)?;
        }
        //match the first character of key
        let first_char = self.key.as_ref().unwrap().as_str().chars().next().unwrap();
        match first_char {
            'e' => {
                state.serialize_field("tier", "Empire of")?;
            }
            'k' => {
                state.serialize_field("tier", "Kingdom of")?;
            }
            'd' => {
                state.serialize_field("tier", "Duchy of")?;
            }
            'c' => {
                state.serialize_field("tier", "County of")?;
            }
            'b' => {
                state.serialize_field("tier", "Barony of")?;
            }
            _ => {
                state.serialize_field("tier", "")?;
            }
        }
        if self.de_facto.is_some() {
            let de_facto = DerivedRef::from_derived(self.de_facto.as_ref().unwrap().clone());
            state.serialize_field("de_facto", &de_facto)?;
        }
        state.serialize_field("de_jure_vassals", &serialize_array(&self.de_jure_vassals))?;
        state.serialize_field("de_facto_vassals", &serialize_array(&self.de_facto_vassals))?;
        let mut history = Vec::new();
        for h in self.history.iter() {
            let mut o = (h.0.clone(), None, h.2.clone());
            if h.1.is_some() {
                let c = DerivedRef::from_derived(h.1.as_ref().unwrap().clone());
                o.1 = Some(c);
            }
            history.push(o);
        }
        state.serialize_field("claims", &serialize_array(&self.claims))?;
        state.serialize_field("history", &history)?;
        if self.capital.is_some() {
            state.serialize_field(
                "capital",
                &DerivedRef::from_derived(self.capital.as_ref().unwrap().clone()),
            )?;
        }
        state.end()
    }
}

impl Renderable for Title {
    fn get_context(&self) -> minijinja::Value {
        return context! {title=>self};
    }

    fn get_template() -> &'static str {
        "titleTemplate.html"
    }

    fn get_subdir() -> &'static str {
        "titles"
    }

    fn render_all(&self, stack: &mut Vec<RenderableType>, renderer: &mut Renderer) {
        if !renderer.render(self) {
            return;
        }
        let game_map = renderer.get_map();
        if game_map.is_some() && self.de_facto_vassals.len() > 0 {
            let map = game_map.unwrap();
            let path = format!("{}/titles/{}.png", renderer.get_path(), self.id);
            map.create_map_file(self.get_barony_keys(), &self.color, &path);
        }
        if self.de_jure.is_some() {
            stack.push(RenderableType::Title(
                self.de_jure.as_ref().unwrap().clone(),
            ));
        }
        if self.de_facto.is_some() {
            stack.push(RenderableType::Title(
                self.de_facto.as_ref().unwrap().clone(),
            ));
        }
        for v in &self.de_jure_vassals {
            stack.push(RenderableType::Title(v.clone()));
        }
        for o in &self.history {
            if o.1.is_some() {
                stack.push(RenderableType::Character(o.1.as_ref().unwrap().clone()));
            }
        }
        if self.capital.is_some() {
            stack.push(RenderableType::Title(
                self.capital.as_ref().unwrap().clone(),
            ));
        }
    }
}

impl Cullable for Title {
    fn set_depth(&mut self, depth: usize, localization: &Localizer) {
        if depth <= self.depth && depth != 0 {
            return;
        }
        if !self.name_localized && self.key.is_some() {
            //localization
            if self.name == self.key {
                self.name = Some(localization.localize(self.key.as_ref().unwrap().as_str()));
            }
            self.name_localized = true;
        }
        if depth == 0 {
            return;
        }
        self.depth = depth;
        if self.de_jure.is_some() {
            let c = self.de_jure.as_ref().unwrap().try_get_internal_mut();
            if c.is_ok() {
                c.unwrap().set_depth(depth - 1, localization);
            }
        }
        if self.de_facto.is_some() {
            let c = self.de_facto.as_ref().unwrap().try_get_internal_mut();
            if c.is_ok() {
                c.unwrap().set_depth(depth - 1, localization);
            }
        }
        for v in &self.de_jure_vassals {
            let o = v.try_get_internal_mut();
            if o.is_ok() {
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
        for v in &self.de_facto_vassals {
            let o = v.try_get_internal_mut();
            if o.is_ok() {
                o.unwrap().set_depth(depth - 1, localization);
            }
        }
        for o in self.history.iter_mut() {
            if !self.localized {
                o.2 = localization.localize(o.2.as_str());
            }
            if o.1.is_some() {
                let c = o.1.as_ref().unwrap().try_get_internal_mut();
                if c.is_ok() {
                    c.unwrap().set_depth(depth - 1, localization);
                }
            }
        }
        if self.capital.is_some() {
            let c = self.capital.as_ref().unwrap().try_get_internal_mut();
            if c.is_ok() {
                let mut c = c.unwrap();
                if c.id != self.id {
                    c.set_depth(depth - 1, localization);
                }
            }
        }
        self.localized = true;
    }

    fn get_depth(&self) -> usize {
        self.depth
    }
}

impl Title {
    pub fn get_holder(&self) -> Option<Shared<Character>> {
        let entry = self.history.last();
        if entry.is_none() {
            return None;
        }
        entry.unwrap().1.clone()
    }
}
