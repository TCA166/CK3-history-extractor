use std::collections::HashMap;

use crate::structures::GameObjectDerived;

use super::{
    super::{
        display::{Grapher, RealmDifference, Timeline},
        game_data::{Localizable, Localize},
        structures::{
            Artifact, Character, Culture, DerivedRef, DummyInit, Dynasty, Faith, Memory, Title,
        },
        types::{RefOrRaw, Shared, Wrapper, WrapperMut},
    },
    game_object::{GameId, GameObjectMap, GameString},
    ParsingError,
};

use serde::Serialize;

/// Returns a reference to the object with the given key in the map, or inserts a dummy object if it does not exist and returns a reference to that.
fn get_or_insert_dummy<T: DummyInit>(
    map: &mut HashMap<GameId, Shared<T>>,
    key: &GameId,
) -> Shared<T> {
    if let Some(val) = map.get(key) {
        return val.clone();
    } else {
        let v = Shared::wrap(T::dummy(*key));
        map.insert(*key, v.clone());
        v
    }
}

/// A struct representing all known game objects.
/// It is guaranteed to always return a reference to the same object for the same key.
/// Naturally the value of that reference may change as values are added to the game state.
/// This is mainly used during the process of gathering data from the parsed save file.
#[derive(Serialize)]
pub struct GameState {
    /// A character id->Character transform
    characters: HashMap<GameId, Shared<Character>>,
    /// A title id->Title transform
    titles: HashMap<GameId, Shared<Title>>,
    /// A faith id->Title transform
    faiths: HashMap<GameId, Shared<Faith>>,
    /// A culture id->Culture transform
    cultures: HashMap<GameId, Shared<Culture>>,
    /// A dynasty id->Dynasty transform
    dynasties: HashMap<GameId, Shared<Dynasty>>,
    /// A memory id->Memory transform
    memories: HashMap<GameId, Shared<Memory>>,
    /// A artifact id->Artifact transform
    artifacts: HashMap<GameId, Shared<Artifact>>,
    /// A trait id->Trait identifier transform
    traits_lookup: Vec<GameString>,
    /// A vassal contract id->Character transform
    contract_transform: HashMap<GameId, Shared<DerivedRef<Character>>>,
    /// The current date from the meta section
    current_date: Option<GameString>,
    /// The isolated year from the meta section
    current_year: Option<u32>,
    /// The date from which data should be considered
    offset_date: Option<u32>,
}

impl GameState {
    /// Create a new GameState
    pub fn new() -> GameState {
        GameState {
            characters: HashMap::default(),
            titles: HashMap::default(),
            faiths: HashMap::default(),
            cultures: HashMap::default(),
            dynasties: HashMap::default(),
            memories: HashMap::default(),
            artifacts: HashMap::default(),
            traits_lookup: Vec::new(),
            contract_transform: HashMap::default(),
            current_date: None,
            current_year: None,
            offset_date: None,
        }
    }

    /// Add a lookup table for traits
    pub fn add_lookup(&mut self, array: Vec<GameString>) {
        self.traits_lookup = array;
    }

    /// Get a trait by id
    pub fn get_trait(&self, id: u16) -> GameString {
        self.traits_lookup[id as usize].clone()
    }

    /// Set the current date
    pub fn set_current_date(&mut self, date: GameString) {
        self.current_date = Some(date.clone());
        self.current_year = Some(date.as_str().split_once('.').unwrap().0.parse().unwrap());
    }

    /// Set the number of years that has passed since game start
    pub fn set_offset_date(&mut self, date: GameString) {
        self.offset_date = Some(date.split_once('.').unwrap().0.parse().unwrap());
    }

    /// Get the current date
    pub fn get_current_date(&self) -> Option<&str> {
        if self.current_date.is_none() {
            return None;
        } else {
            return Some(self.current_date.as_ref().unwrap().as_str());
        }
    }

    /// Get a character by key
    pub fn get_character(&mut self, key: &GameId) -> Shared<Character> {
        get_or_insert_dummy(&mut self.characters, key)
    }

    /// Gets the vassal associated with the contract with the given id
    pub fn get_vassal(&mut self, contract_id: &GameId) -> Shared<DerivedRef<Character>> {
        if !self.contract_transform.contains_key(contract_id) {
            let v = Shared::wrap(DerivedRef::dummy());
            self.contract_transform.insert(*contract_id, v.clone());
            v
        } else {
            self.contract_transform.get(contract_id).unwrap().clone()
        }
    }

    /// Adds a new vassal contract
    pub fn add_contract(&mut self, contract_id: &GameId, character_id: &GameId) {
        let char = self.get_character(character_id);
        if self.contract_transform.contains_key(contract_id) {
            let entry = self.contract_transform.get(contract_id).unwrap();
            entry.get_internal_mut().init(char);
        } else {
            let r = Shared::wrap(DerivedRef::from(char));
            self.contract_transform.insert(*contract_id, r);
        }
    }

    /// Get a title by key
    pub fn get_title(&mut self, key: &GameId) -> Shared<Title> {
        get_or_insert_dummy(&mut self.titles, key)
    }

    /// Get a faith by key
    pub fn get_faith(&mut self, key: &GameId) -> Shared<Faith> {
        get_or_insert_dummy(&mut self.faiths, key)
    }

    /// Get a culture by key
    pub fn get_culture(&mut self, key: &GameId) -> Shared<Culture> {
        get_or_insert_dummy(&mut self.cultures, key)
    }

    /// Get a dynasty by key
    pub fn get_dynasty(&mut self, key: &GameId) -> Shared<Dynasty> {
        get_or_insert_dummy(&mut self.dynasties, key)
    }

    /// Get a memory by key
    pub fn get_memory(&mut self, key: &GameId) -> Shared<Memory> {
        get_or_insert_dummy(&mut self.memories, key)
    }

    /// Get an artifact by key
    pub fn get_artifact(&mut self, key: &GameId) -> Shared<Artifact> {
        get_or_insert_dummy(&mut self.artifacts, key)
    }

    pub fn add_artifact(
        &mut self,
        key: &GameId,
        value: &GameObjectMap,
    ) -> Result<(), ParsingError> {
        get_or_insert_dummy(&mut self.artifacts, key)
            .get_internal_mut()
            .init(value, self)
    }

    /// Add a character to the game state    
    pub fn add_character(
        &mut self,
        key: &GameId,
        value: &GameObjectMap,
    ) -> Result<(), ParsingError> {
        get_or_insert_dummy(&mut self.characters, key)
            .get_internal_mut()
            .init(value, self)
    }

    /// Add a title to the game state
    pub fn add_title(&mut self, key: &GameId, value: &GameObjectMap) -> Result<(), ParsingError> {
        get_or_insert_dummy(&mut self.titles, key)
            .get_internal_mut()
            .init(value, self)
    }

    /// Add a faith to the game state
    pub fn add_faith(&mut self, key: &GameId, value: &GameObjectMap) -> Result<(), ParsingError> {
        get_or_insert_dummy(&mut self.faiths, key)
            .get_internal_mut()
            .init(value, self)
    }

    /// Add a culture to the game state
    pub fn add_culture(&mut self, key: &GameId, value: &GameObjectMap) -> Result<(), ParsingError> {
        get_or_insert_dummy(&mut self.cultures, key)
            .get_internal_mut()
            .init(value, self)
    }

    /// Add a dynasty to the game state
    pub fn add_dynasty(&mut self, key: &GameId, value: &GameObjectMap) -> Result<(), ParsingError> {
        get_or_insert_dummy(&mut self.dynasties, key)
            .get_internal_mut()
            .init(value, self)
    }

    /// Add a memory to the game state
    pub fn add_memory(&mut self, key: &GameId, value: &GameObjectMap) -> Result<(), ParsingError> {
        get_or_insert_dummy(&mut self.memories, key)
            .get_internal_mut()
            .init(value, self)
    }

    pub fn get_baronies_of_counties<F: Fn(&RefOrRaw<Title>) -> bool>(
        &self,
        filter: F,
    ) -> Vec<GameString> {
        let mut res = Vec::new();
        for title in self.titles.values() {
            let title = title.get_internal();
            if filter(&title) {
                res.append(&mut title.get_barony_keys());
            }
        }
        res
    }

    pub fn add_county_data(
        &mut self,
        county_data: HashMap<&str, (Shared<Faith>, Shared<Culture>)>,
    ) {
        for title in self.titles.values() {
            let key = title.get_internal().get_key();
            if key.is_none() {
                continue;
            }
            let assoc = county_data.get(key.unwrap().as_str());
            if assoc.is_none() {
                continue;
            }
            let (faith, culture) = assoc.unwrap();
            title
                .get_internal_mut()
                .add_county_data(culture.clone(), faith.clone())
        }
    }

    pub fn new_grapher(&self) -> Grapher {
        let mut total_yearly_deaths: HashMap<u32, i32> = HashMap::default();
        let mut faith_yearly_deaths = HashMap::default();
        let mut culture_yearly_deaths = HashMap::default();
        for character in self.characters.values() {
            let char = character.get_internal();
            if let Some(death_date) = char.get_death_date() {
                let death_year: u32 = death_date.split_once('.').unwrap().0.parse().unwrap();
                let count = total_yearly_deaths.entry(death_year).or_insert(0);
                *count += 1;
                if let Some(faith) = char.get_faith() {
                    let entry = faith_yearly_deaths
                        .entry(faith.get_internal().get_id())
                        .or_insert(HashMap::default());
                    let count = entry.entry(death_year).or_insert(0.);
                    *count += 1.;
                }
                if let Some(culture) = char.get_culture() {
                    let entry = culture_yearly_deaths
                        .entry(culture.get_internal().get_id())
                        .or_insert(HashMap::default());
                    let count = entry.entry(death_year).or_insert(0.);
                    *count += 1.;
                }
            }
        }
        for (year, tot) in total_yearly_deaths {
            for data in faith_yearly_deaths.values_mut() {
                if let Some(count) = data.get_mut(&year) {
                    *count /= tot as f64;
                }
            }
            for data in culture_yearly_deaths.values_mut() {
                if let Some(count) = data.get_mut(&year) {
                    *count /= tot as f64;
                }
            }
        }
        Grapher::new(faith_yearly_deaths, culture_yearly_deaths)
    }

    pub fn new_timeline(&self) -> Timeline {
        const DESTROYED_STR: &str = "destroyed";
        const USURPED_STR: &str = "usurped";
        const CONQUERED_START_STR: &str = "conq"; //this should match both 'conquered' and 'conquest holy war'

        let mut lifespans = Vec::new();
        let mut latest_event = 0;
        let mut event_checkout = Vec::new();
        for title in self.titles.values() {
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
        return Timeline::new(lifespans, self.current_year.unwrap(), events);
    }
}

impl Localizable for GameState {
    fn localize<L: Localize>(&mut self, localization: &mut L) {
        for (_, character) in &mut self.characters {
            character.get_internal_mut().localize(localization);
        }
        for (_, title) in &mut self.titles {
            title.get_internal_mut().localize(localization);
        }
        for (_, faith) in &mut self.faiths {
            faith.get_internal_mut().localize(localization);
        }
        for (_, culture) in &mut self.cultures {
            culture.get_internal_mut().localize(localization);
        }
        for (_, dynasty) in &mut self.dynasties {
            dynasty.get_internal_mut().localize(localization);
        }
        for (_, memory) in &mut self.memories {
            memory.get_internal_mut().localize(localization);
        }
        for (_, artifact) in &mut self.artifacts {
            artifact.get_internal_mut().localize(localization);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::structures::GameObjectDerived;

    use super::*;

    #[test]
    fn test_get_or_insert_dummy() {
        let mut map = HashMap::default();
        let key = 1;
        let val = get_or_insert_dummy::<Artifact>(&mut map, &key);
        assert_eq!(val.get_internal().get_id(), key);
        let val2 = get_or_insert_dummy(&mut map, &key);
        assert_eq!(val.get_internal().get_id(), val2.get_internal().get_id());
    }
}
