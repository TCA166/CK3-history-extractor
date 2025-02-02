use std::slice::Iter;
use std::{cmp::Ordering, path::Path};

use jomini::common::PdsDate;
use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        display::{Cullable, Grapher, Renderable, RenderableType, TreeNode},
        game_data::{GameData, Localizable, Localize, MapGenerator},
        jinja_env::TITLE_TEMPLATE_NAME,
        parser::{
            GameId, GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError,
            SaveFileObject, SaveFileValue,
        },
        types::{OneOrMany, Wrapper, WrapperMut},
    },
    derived_ref::into_ref_array,
    Character, Culture, DerivedRef, DummyInit, Faith, GameObjectDerived, Shared,
};

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
    capital: Option<Shared<Title>>,
    /// Only used for counties
    culture: Option<Shared<Culture>>,
    /// Only used for counties
    faith: Option<Shared<Faith>>,
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

    pub fn get_de_jure_barony_keys(&self) -> Vec<GameString> {
        let mut provinces = Vec::new();
        if self.key.as_ref().unwrap().starts_with("b_") {
            provinces.push(self.key.clone().unwrap());
        }
        for v in &self.de_jure_vassals {
            provinces.append(&mut v.get_internal().get_barony_keys());
        }
        provinces
    }

    /// Returns the key of the title
    pub fn get_key(&self) -> Option<GameString> {
        self.key.clone()
    }

    /// Returns an iterator over the history of the title
    pub fn get_history_iter(&self) -> Iter<(GameString, Option<Shared<Character>>, GameString)> {
        self.history.iter()
    }

    /// Returns the capital of the title
    pub fn get_capital(&self) -> Option<Shared<Title>> {
        self.capital.clone()
    }

    /// Adds the culture and faith data to the title
    pub fn add_county_data(&mut self, culture: Shared<Culture>, faith: Shared<Faith>) {
        if !self.key.as_ref().unwrap().starts_with("c_") {
            panic!("Can only add county data to a county title");
        }
        self.culture = Some(culture);
        self.faith = Some(faith);
    }

    /// Returns the culture of the title
    pub fn get_culture(&self) -> Option<Shared<Culture>> {
        if let Some(culture) = &self.culture {
            return Some(culture.clone());
        } else {
            return None;
        }
    }

    /// Returns the faith of the title
    pub fn get_faith(&self) -> Option<Shared<Faith>> {
        if let Some(faith) = &self.faith {
            return Some(faith.clone());
        } else {
            return None;
        }
    }

    /// Returns the holder of the title
    pub fn get_holder(&self) -> Option<Shared<Character>> {
        if let Some(entry) = self.history.last() {
            return entry.1.clone();
        }
        None
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
            color: [70, 255, 70],
            capital: None,
            culture: None,
            faith: None,
        }
    }

    fn init(
        &mut self,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<(), ParsingError> {
        if let Some(color) = base.get("color") {
            let color = color.as_object()?.as_array()?;
            self.color = [
                color[0].as_integer()? as u8,
                color[1].as_integer()? as u8,
                color[2].as_integer()? as u8,
            ];
        }
        self.key = Some(base.get_string("key")?);
        if let Some(de_jure_id) = base.get("de_jure_liege") {
            let o = game_state.get_title(&de_jure_id.as_id()?).clone();
            o.get_internal_mut()
                .add_jure_vassal(game_state.get_title(&self.id).clone());
            self.de_jure = Some(o);
        }
        if let Some(de_facto_id) = base.get("de_facto_liege") {
            let o = game_state.get_title(&de_facto_id.as_id()?).clone();
            o.get_internal_mut()
                .add_facto_vassal(game_state.get_title(&self.id).clone());
            self.de_facto = Some(o);
        }
        if let Some(claims) = base.get("claim") {
            if let SaveFileValue::Object(claims) = claims {
                for claim in claims.as_array()? {
                    self.claims
                        .push(game_state.get_character(&claim.as_id()?).clone());
                }
            } else {
                self.claims
                    .push(game_state.get_character(&claims.as_id()?).clone());
            }
        }
        if let Some(capital) = base.get("capital") {
            self.capital = Some(game_state.get_title(&capital.as_id()?).clone());
        }
        self.name = Some(base.get_string("name")?);
        if let Some(hist) = base.get("history") {
            let hist_obj = hist.as_object()?.as_map()?;
            for (h, val) in hist_obj {
                let character;
                let action: GameString;
                if let SaveFileValue::Object(o) = val {
                    match o {
                        SaveFileObject::Array(arr) => {
                            for entry in arr {
                                let loc_action;
                                let loc_character;
                                if let SaveFileValue::Object(o) = entry {
                                    let o = o.as_map()?;
                                    loc_action = o.get_string("type")?;
                                    if let Some(holder) = o.get("holder") {
                                        loc_character = Some(
                                            game_state.get_character(&holder.as_id()?).clone(),
                                        );
                                    } else {
                                        loc_character = None;
                                    }
                                } else {
                                    loc_action = GameString::wrap("Inherited".to_owned());
                                    loc_character =
                                        Some(game_state.get_character(&entry.as_id()?).clone());
                                }
                                self.history.push((
                                    GameString::wrap(h.to_string()),
                                    loc_character,
                                    loc_action,
                                ))
                            }
                            continue; //if it's an array we handled all the adding already in the loop above
                        }
                        SaveFileObject::Map(o) => {
                            action = o.get_string("type")?;
                            let holder = o.get("holder");
                            match holder {
                                Some(h) => {
                                    character = Some(game_state.get_character(&h.as_id()?).clone());
                                }
                                None => {
                                    character = None;
                                }
                            }
                        }
                    }
                } else {
                    action = GameString::wrap("Inherited".to_owned());
                    character = Some(game_state.get_character(&val.as_id()?).clone());
                }
                self.history
                    .push((GameString::wrap(h.to_string()), character, action));
            }
        }
        //sort history by the first element of the tuple (the date) in descending order
        self.history
            .sort_by(|a, b| date_string_cmp(a.0.as_str(), b.0.as_str()));
        Ok(())
    }
}

impl GameObjectDerived for Title {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        if let Some(name) = &self.name {
            return name.clone();
        } else {
            return GameString::wrap("Unnamed".to_owned());
        }
    }
}

impl TreeNode for Title {
    fn get_children(&self) -> Option<OneOrMany<Self>> {
        if self.de_jure_vassals.is_empty() {
            return None;
        }
        Some(OneOrMany::Many(&self.de_jure_vassals))
    }

    fn get_class(&self) -> Option<GameString> {
        if self.key.is_none() {
            return None;
        }
        let first_char = self.key.as_ref().unwrap().as_str().chars().next().unwrap();
        match first_char {
            'e' => {
                return Some(GameString::wrap("Empire".to_owned()));
            }
            'k' => {
                return Some(GameString::wrap("Kingdom".to_owned()));
            }
            'd' => {
                return Some(GameString::wrap("Duchy".to_owned()));
            }
            'c' => {
                return Some(GameString::wrap("County".to_owned()));
            }
            'b' => {
                return Some(GameString::wrap("Barony".to_owned()));
            }
            _ => {
                return None;
            }
        }
    }

    fn get_parent(&self) -> Option<OneOrMany<Self>> {
        if let Some(de_jure) = &self.de_jure {
            return Some(OneOrMany::One(de_jure));
        }
        None
    }
}

impl Serialize for Title {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Title", 12)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        if let Some(tier) = self.get_class() {
            state.serialize_field("tier", &tier)?;
        } else {
            state.serialize_field("tier", "")?;
        }
        if let Some(faith) = &self.faith {
            let faith = DerivedRef::from(faith.clone());
            state.serialize_field("faith", &faith)?;
        }
        if let Some(culture) = &self.culture {
            let culture = DerivedRef::from(culture.clone());
            state.serialize_field("culture", &culture)?;
        }
        if let Some(de_jure) = &self.de_jure {
            let de_jure = DerivedRef::from(de_jure.clone());
            state.serialize_field("de_jure", &de_jure)?;
        }
        if let Some(de_facto) = &self.de_facto {
            let de_facto = DerivedRef::from(de_facto.clone());
            state.serialize_field("de_facto", &de_facto)?;
        }
        state.serialize_field("de_jure_vassals", &into_ref_array(&self.de_jure_vassals))?;
        state.serialize_field("de_facto_vassals", &into_ref_array(&self.de_facto_vassals))?;
        let mut history = Vec::new();
        for h in self.history.iter() {
            let mut o = (h.0.clone(), None, h.2.clone());
            if let Some(holder) = &h.1 {
                let c = DerivedRef::from(holder.clone());
                o.1 = Some(c);
            }
            history.push(o);
        }
        state.serialize_field("claims", &into_ref_array(&self.claims))?;
        state.serialize_field("history", &history)?;
        if let Some(capital) = &self.capital {
            state.serialize_field("capital", &DerivedRef::from(capital.clone()))?;
        }
        state.end()
    }
}

impl Renderable for Title {
    fn get_template() -> &'static str {
        TITLE_TEMPLATE_NAME
    }

    fn get_subdir() -> &'static str {
        "titles"
    }

    fn append_ref(&self, stack: &mut Vec<RenderableType>) {
        if let Some(de_jure) = &self.de_jure {
            stack.push(RenderableType::Title(de_jure.clone()));
        }
        if let Some(de_facto) = &self.de_facto {
            stack.push(RenderableType::Title(de_facto.clone()));
        }
        for v in &self.de_jure_vassals {
            stack.push(RenderableType::Title(v.clone()));
        }
        for o in &self.history {
            if let Some(character) = &o.1 {
                stack.push(RenderableType::Character(character.clone()));
            }
        }
        if let Some(capital) = &self.capital {
            stack.push(RenderableType::Title(capital.clone()));
        }
    }

    fn render(&self, path: &Path, game_state: &GameState, _: Option<&Grapher>, data: &GameData) {
        if let Some(map) = data.get_map() {
            if self.de_facto_vassals.len() == 0 {
                return;
            }
            let mut buf = path.join(Self::get_subdir());
            buf.push(format!("{}.png", self.id));
            let mut title_map = map.create_map_flat(self.get_barony_keys(), self.color);
            title_map.draw_text(format!(
                "{} at {}",
                self.name.as_ref().unwrap(),
                game_state.get_current_date().unwrap().iso_8601()
            ));
            title_map.save(&buf);
        }
    }
}

impl Localizable for Title {
    fn localize<L: Localize>(&mut self, localization: &mut L) {
        if self.key.is_none() {
            return;
        }
        if self.name == self.key {
            self.name = Some(localization.localize(self.key.as_ref().unwrap().as_str()));
        }
        //for o in self.history.iter_mut() {
        //    o.2 = localization.localize(o.2.as_str());
        //}
    }
}

impl Cullable for Title {
    fn set_depth(&mut self, depth: usize) {
        if depth <= self.depth {
            return;
        }
        self.depth = depth;
        let depth = depth - 1;
        if let Some(de_jure) = &self.de_jure {
            if let Ok(mut c) = de_jure.try_get_internal_mut() {
                c.set_depth(depth);
            }
        }
        if let Some(de_facto) = &self.de_facto {
            if let Ok(mut c) = de_facto.try_get_internal_mut() {
                c.set_depth(depth);
            }
        }
        for v in &self.de_jure_vassals {
            if let Ok(mut v) = v.try_get_internal_mut() {
                v.set_depth(depth);
            }
        }
        for v in &self.de_facto_vassals {
            if let Ok(mut v) = v.try_get_internal_mut() {
                v.set_depth(depth);
            }
        }
        for o in self.history.iter_mut() {
            if let Some(character) = &o.1 {
                if let Ok(mut c) = character.try_get_internal_mut() {
                    c.set_depth(depth);
                }
            }
        }
        if let Some(capital) = &self.capital {
            if let Ok(mut c) = capital.try_get_internal_mut() {
                if c.id != self.id {
                    c.set_depth(depth);
                }
            }
        }
        if let Some(faith) = &self.faith {
            if let Ok(mut f) = faith.try_get_internal_mut() {
                f.set_depth(depth);
            }
        }
        if let Some(culture) = &self.culture {
            if let Ok(mut c) = culture.try_get_internal_mut() {
                c.set_depth(depth);
            }
        }
    }

    fn get_depth(&self) -> usize {
        self.depth
    }
}
