use super::{
    super::{
        game_data::{Localizable, Localize},
        structures::{
            Artifact, Character, Culture, DerivedRef, DummyInit, Dynasty, Faith, Memory, Title,
        },
        types::{HashMap, HashMapIter, RefOrRaw, Shared, Wrapper, WrapperMut},
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

/// Just like [get_or_insert_dummy], but takes a [GameObjectMap] as input and uses the name field as the key.
fn get_or_insert_dummy_from_value<T: DummyInit>(
    map: &mut HashMap<GameId, Shared<T>>,
    value: &GameObjectMap,
) -> Shared<T> {
    let key = value.get_name().parse::<GameId>().unwrap();
    return get_or_insert_dummy(map, &key);
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

    pub fn add_artifact(&mut self, value: &GameObjectMap) -> Result<(), ParsingError> {
        get_or_insert_dummy_from_value(&mut self.artifacts, value)
            .get_internal_mut()
            .init(value, self)
    }

    /// Add a character to the game state    
    pub fn add_character(&mut self, value: &GameObjectMap) -> Result<(), ParsingError> {
        get_or_insert_dummy_from_value(&mut self.characters, value)
            .get_internal_mut()
            .init(value, self)
    }

    /// Add a title to the game state
    pub fn add_title(&mut self, value: &GameObjectMap) -> Result<(), ParsingError> {
        get_or_insert_dummy_from_value(&mut self.titles, value)
            .get_internal_mut()
            .init(value, self)
    }

    /// Add a faith to the game state
    pub fn add_faith(&mut self, value: &GameObjectMap) -> Result<(), ParsingError> {
        get_or_insert_dummy_from_value(&mut self.faiths, value)
            .get_internal_mut()
            .init(value, self)
    }

    /// Add a culture to the game state
    pub fn add_culture(&mut self, value: &GameObjectMap) -> Result<(), ParsingError> {
        get_or_insert_dummy_from_value(&mut self.cultures, value)
            .get_internal_mut()
            .init(value, self)
    }

    /// Add a dynasty to the game state
    pub fn add_dynasty(&mut self, value: &GameObjectMap) -> Result<(), ParsingError> {
        get_or_insert_dummy_from_value(&mut self.dynasties, value)
            .get_internal_mut()
            .init(value, self)
    }

    /// Add a memory to the game state
    pub fn add_memory(&mut self, value: &GameObjectMap) -> Result<(), ParsingError> {
        get_or_insert_dummy_from_value(&mut self.memories, value)
            .get_internal_mut()
            .init(value, self)
    }

    /// Creates a hashmap death year->number of deaths
    pub fn get_total_yearly_deaths(&self) -> HashMap<u32, u32> {
        let mut result = HashMap::default();
        for (_, character) in &self.characters {
            let char = character.get_internal();
            let death_date = char.get_death_date();
            if death_date.is_none() {
                continue;
            }
            let death_date = death_date.unwrap();
            let death_year: u32 = death_date.split_once('.').unwrap().0.parse().unwrap();
            if let Some(offset) = self.offset_date {
                if death_year < self.current_year.unwrap() - offset {
                    continue;
                }
            }
            let count = result.entry(death_year).or_insert(0);
            *count += 1;
        }
        return result;
    }

    /// Returns a hashmap of classes of characters and their associated yearly death graphs
    /// So for example if you provide a function that returns the dynasty of a character
    /// you will get a hashmap of dynasties and their yearly death counts
    pub fn get_yearly_deaths<F>(
        &self,
        associate: F,
        total: &HashMap<u32, u32>,
    ) -> HashMap<GameId, Vec<(u32, f64)>>
    where
        F: Fn(RefOrRaw<Character>) -> Option<GameId>,
    {
        let mut result = HashMap::default();
        for (_, character) in &self.characters {
            let char = character.get_internal();
            let death_date = char.get_death_date();
            if death_date.is_none() {
                continue;
            }
            let key = associate(char);
            if key.is_none() {
                continue;
            }
            let death_date = death_date.unwrap();
            let death_year: u32 = death_date.split_once('.').unwrap().0.parse().unwrap();
            if let Some(offset) = self.offset_date {
                if death_year < self.current_year.unwrap() - offset {
                    continue;
                }
            }
            let entry = result.entry(key.unwrap()).or_insert(HashMap::default());
            let count = entry.entry(death_year).or_insert(0);
            *count += 1;
        }
        // convert the internal hashmaps to vectors
        let mut res = HashMap::default();
        for (id, data) in result {
            let mut v = Vec::new();
            for (year, count) in &data {
                v.push((*year, *count as f64 / *total.get(year).unwrap() as f64));
            }
            let max_yr = data.keys().max().unwrap();
            for yr in 0..=*max_yr {
                if !data.contains_key(&yr)
                    && ((yr != 0 && data.contains_key(&(yr - 1))) || data.contains_key(&(yr + 1)))
                {
                    v.push((yr, 0.0));
                }
            }
            v.sort_by(|a, b| a.0.cmp(&b.0));
            res.insert(id, v);
        }
        return res;
    }

    /// Returns a iterator over the titles
    pub fn get_title_iter(&self) -> HashMapIter<GameId, Shared<Title>> {
        self.titles.iter()
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

    #[test]
    fn test_get_or_insert_dummy_from_value() {
        let mut map = HashMap::default();
        let key = 1;
        let mut value = GameObjectMap::new();
        value.rename("1".to_owned());
        let val = get_or_insert_dummy_from_value::<Artifact>(&mut map, &value);
        assert_eq!(val.get_internal().get_id(), key);
        let val2 = get_or_insert_dummy(&mut map, &key);
        assert_eq!(val.get_internal().get_id(), val2.get_internal().get_id());
    }
}
