use std::collections::{BTreeMap, HashMap};

use super::{
    super::{
        display::{Grapher, RealmDifference, Timeline},
        game_data::{Localizable, LocalizationError, Localize},
        structures::{
            Artifact, Character, Culture, Dynasty, Faith, FromGameObject, GameObjectDerived,
            GameObjectEntity, House, Memory, Title,
        },
        types::{GameId, GameString, Shared, Wrapper, WrapperMut},
    },
    game_object::GameObjectMap,
    ParsingError,
};

use jomini::common::{Date, PdsDate};

use serde::{ser::SerializeMap, Serialize, Serializer};

pub type GameRef<T> = Shared<GameObjectEntity<T>>;

/// Returns a reference to the object with the given key in the map, or inserts a dummy object if it does not exist and returns a reference to that.
fn get_or_insert_dummy<T: GameObjectDerived + FromGameObject>(
    map: &mut HashMap<GameId, GameRef<T>>,
    key: &GameId,
) -> GameRef<T> {
    if let Some(val) = map.get(key) {
        return val.clone();
    } else {
        let v = Shared::wrap(GameObjectEntity::new(*key));
        map.insert(*key, v.clone());
        v
    }
}

impl Serialize for Shared<Option<GameRef<Character>>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.get_internal().as_ref() {
            Some(c) => c.serialize(serializer),
            None => serializer.serialize_none(),
        }
    }
}

fn serialize_ref_map<T: Serialize + GameObjectDerived, S: Serializer>(
    map: &HashMap<GameId, GameRef<T>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut state = serializer.serialize_map(Some(map.len()))?;
    for (k, v) in map.iter() {
        state.serialize_entry(k, &*v.get_internal())?;
    }
    state.end()
}

/// A struct representing all known game objects.
/// It is guaranteed to always return a reference to the same object for the same key.
/// Naturally the value of that reference may change as values are added to the game state.
/// This is mainly used during the process of gathering data from the parsed save file.
#[derive(Serialize)]
pub struct GameState {
    /// A character id->Character transform
    #[serde(serialize_with = "serialize_ref_map")]
    characters: HashMap<GameId, GameRef<Character>>,
    /// A title id->Title transform
    #[serde(serialize_with = "serialize_ref_map")]
    titles: HashMap<GameId, GameRef<Title>>,
    /// A faith id->Title transform
    #[serde(serialize_with = "serialize_ref_map")]
    faiths: HashMap<GameId, GameRef<Faith>>,
    /// A culture id->Culture transform
    #[serde(serialize_with = "serialize_ref_map")]
    cultures: HashMap<GameId, GameRef<Culture>>,
    /// A dynasty id->Dynasty transform
    #[serde(serialize_with = "serialize_ref_map")]
    dynasties: HashMap<GameId, GameRef<Dynasty>>,
    #[serde(serialize_with = "serialize_ref_map")]
    houses: HashMap<GameId, GameRef<House>>,
    /// A memory id->Memory transform
    #[serde(serialize_with = "serialize_ref_map")]
    memories: HashMap<GameId, GameRef<Memory>>,
    /// A artifact id->Artifact transform
    #[serde(serialize_with = "serialize_ref_map")]
    artifacts: HashMap<GameId, GameRef<Artifact>>,
    /// A trait id->Trait identifier transform
    traits_lookup: Vec<GameString>,
    /// A vassal contract id->Character transform
    contract_transform: HashMap<GameId, Shared<Option<GameRef<Character>>>>,
    character_transform: HashMap<GameId, GameId>,
    #[serde(skip)]
    county_data: HashMap<String, (GameRef<Faith>, GameRef<Culture>)>,
    /// The current date from the meta section
    current_date: Option<Date>,
    /// The time Y.M.D from which the game started
    offset_date: Option<Date>,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            characters: HashMap::default(),
            titles: HashMap::default(),
            faiths: HashMap::default(),
            cultures: HashMap::default(),
            dynasties: HashMap::default(),
            houses: HashMap::default(),
            memories: HashMap::default(),
            artifacts: HashMap::default(),
            traits_lookup: Vec::new(),
            contract_transform: HashMap::default(),
            character_transform: HashMap::default(),
            county_data: HashMap::default(),
            current_date: None,
            offset_date: None,
        }
    }
}

impl GameState {
    /// Add a lookup table for traits
    pub fn add_lookup(&mut self, array: Vec<GameString>) {
        self.traits_lookup = array;
    }

    pub fn add_character_transform(&mut self, transform: HashMap<GameId, GameId>) {
        self.character_transform = transform;
    }

    /// Get a trait by id
    pub fn get_trait(&self, id: u16) -> GameString {
        self.traits_lookup[id as usize].clone()
    }

    /// Set the current date
    pub fn set_current_date(&mut self, date: Date, offset: Date) {
        self.current_date = Some(date);
        self.offset_date = Some(offset);
    }

    /// Get the current date
    pub fn get_current_date(&self) -> Option<Date> {
        return self.current_date;
    }

    /// Get a character by key
    pub fn get_character(&mut self, key: &GameId) -> GameRef<Character> {
        get_or_insert_dummy(&mut self.characters, key)
    }

    /// Gets the vassal associated with the contract with the given id
    pub fn get_vassal(&mut self, contract_id: &GameId) -> Shared<Option<GameRef<Character>>> {
        if let Some(v) = self.contract_transform.get(contract_id) {
            return v.clone();
        } else {
            let v = Shared::wrap(None);
            self.contract_transform.insert(*contract_id, v.clone());
            return v;
        }
    }

    /// Adds a new vassal contract
    pub fn add_contract(&mut self, contract_id: &GameId, character_id: &GameId) {
        let char = self.get_character(character_id);
        if let Some(contract) = self.contract_transform.get_mut(contract_id) {
            contract.get_internal_mut().replace(char);
        } else {
            self.contract_transform
                .insert(*contract_id, Shared::wrap(Some(char)));
        }
    }

    /// Get a title by key
    pub fn get_title(&mut self, key: &GameId) -> GameRef<Title> {
        get_or_insert_dummy(&mut self.titles, key)
    }

    /// Get a faith by key
    pub fn get_faith(&mut self, key: &GameId) -> GameRef<Faith> {
        get_or_insert_dummy(&mut self.faiths, key)
    }

    /// Get a culture by key
    pub fn get_culture(&mut self, key: &GameId) -> GameRef<Culture> {
        get_or_insert_dummy(&mut self.cultures, key)
    }

    /// Get a dynasty by key
    pub fn get_dynasty(&mut self, key: &GameId) -> GameRef<Dynasty> {
        get_or_insert_dummy(&mut self.dynasties, key)
    }

    /// Get a memory by key
    pub fn get_memory(&mut self, key: &GameId) -> GameRef<Memory> {
        get_or_insert_dummy(&mut self.memories, key)
    }

    /// Get an artifact by key
    pub fn get_artifact(&mut self, key: &GameId) -> GameRef<Artifact> {
        get_or_insert_dummy(&mut self.artifacts, key)
    }

    pub fn get_house(&mut self, key: &GameId) -> GameRef<House> {
        get_or_insert_dummy(&mut self.houses, key)
    }

    pub fn add_house(&mut self, key: &GameId, value: &GameObjectMap) -> Result<(), ParsingError> {
        self.get_house(key).get_internal_mut().init(value, self)
    }

    pub fn add_artifact(
        &mut self,
        key: &GameId,
        value: &GameObjectMap,
    ) -> Result<(), ParsingError> {
        self.get_artifact(key).get_internal_mut().init(value, self)
    }

    /// Add a character to the game state    
    pub fn add_character(
        &mut self,
        key: &GameId,
        value: &GameObjectMap,
    ) -> Result<(), ParsingError> {
        let char = self.get_character(key);
        char.get_internal_mut().init(value, self)?;
        if let Some(alt) = self.character_transform.get(key) {
            if let Some(alt_char) = self.characters.get(alt) {
                if alt_char.get_internal().inner().is_none() {
                    // TODO this clone is very wasteful
                    alt_char
                        .get_internal_mut()
                        .replace(char.get_internal().inner().unwrap().clone());
                }
            }
        }
        Ok(())
    }

    /// Add a title to the game state
    pub fn add_title(&mut self, key: &GameId, value: &GameObjectMap) -> Result<(), ParsingError> {
        self.get_title(key).get_internal_mut().init(value, self)
    }

    /// Add a faith to the game state
    pub fn add_faith(&mut self, key: &GameId, value: &GameObjectMap) -> Result<(), ParsingError> {
        self.get_faith(key).get_internal_mut().init(value, self)
    }

    /// Add a culture to the game state
    pub fn add_culture(&mut self, key: &GameId, value: &GameObjectMap) -> Result<(), ParsingError> {
        self.get_culture(key).get_internal_mut().init(value, self)
    }

    /// Add a dynasty to the game state
    pub fn add_dynasty(&mut self, key: &GameId, value: &GameObjectMap) -> Result<(), ParsingError> {
        self.get_dynasty(key).get_internal_mut().init(value, self)
    }

    /// Add a memory to the game state
    pub fn add_memory(&mut self, key: &GameId, value: &GameObjectMap) -> Result<(), ParsingError> {
        self.get_memory(key).get_internal_mut().init(value, self)
    }

    pub fn get_baronies_of_counties<F: Fn(&Title) -> bool>(&self, filter: F) -> Vec<GameString> {
        let mut res = Vec::new();
        for title in self.titles.values() {
            if let Some(title) = title.get_internal().inner() {
                if filter(&title) {
                    res.append(&mut title.get_barony_keys());
                }
            }
        }
        res
    }

    pub fn add_county_data(
        &mut self,
        county_data: HashMap<String, (GameRef<Faith>, GameRef<Culture>)>,
    ) {
        self.county_data = county_data;
    }

    pub fn new_grapher(&self) -> Grapher {
        let mut total_yearly_deaths: BTreeMap<i16, u32> = BTreeMap::default();
        let mut faith_yearly_deaths = HashMap::default();
        let mut culture_yearly_deaths = HashMap::default();
        let start_year = self.current_date.unwrap().year() - self.offset_date.unwrap().year();
        for character in self.characters.values() {
            if let Some(char) = character.get_internal().inner() {
                if let Some(death_date) = char.get_death_date() {
                    if death_date.year() <= start_year
                        || death_date.year() >= self.current_date.unwrap().year()
                    {
                        continue;
                    }
                    let count = total_yearly_deaths.entry(death_date.year()).or_insert(0);
                    *count += 1;
                    {
                        let entry = faith_yearly_deaths
                            .entry(char.get_faith().as_ref().unwrap().get_internal().get_id())
                            .or_insert(BTreeMap::default());
                        let count = entry.entry(death_date.year()).or_insert(0);
                        *count += 1;
                    }
                    {
                        let entry = culture_yearly_deaths
                            .entry(char.get_culture().as_ref().unwrap().get_internal().get_id())
                            .or_insert(BTreeMap::default());
                        let count = entry.entry(death_date.year()).or_insert(0);
                        *count += 1;
                    }
                }
            }
        }
        Grapher::new(
            faith_yearly_deaths,
            culture_yearly_deaths,
            total_yearly_deaths,
        )
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
            if let Some(t) = title.get_internal().inner() {
                let hist = t.get_history_iter();
                if hist.len() == 0 {
                    continue;
                }
                let k = t.get_key();
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
                event_checkout.push(t.get_capital().unwrap().clone());
                let mut item = (title.clone(), Vec::new());
                let mut empty = true;
                let mut start = 0;
                for entry in hist {
                    let yr = entry.0.year();
                    if yr > latest_event {
                        latest_event = yr;
                    }
                    let event = entry.2.as_ref();
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
            i16,
            GameRef<Character>,
            GameRef<Title>,
            GameString,
            RealmDifference,
        )> = Vec::new();
        for title in event_checkout {
            if let Some(tit) = title.get_internal().inner() {
                //find the first event that has a character attached
                let mut hist = tit.get_history_iter().skip_while(|a| a.1.is_none());
                let next = hist.next();
                if next.is_none() {
                    continue;
                }
                if let Some(first_char) = next.unwrap().1.as_ref().unwrap().get_internal().inner() {
                    let mut faith = first_char
                        .get_faith()
                        .as_ref()
                        .unwrap()
                        .get_internal()
                        .get_id();
                    let mut culture = first_char
                        .get_culture()
                        .as_ref()
                        .unwrap()
                        .get_internal()
                        .get_id();
                    for entry in hist {
                        let char = entry.1.as_ref();
                        if char.is_none() {
                            continue;
                        }
                        let char = char.unwrap();
                        let event = entry.2.as_ref();
                        if let Some(ch) = char.get_internal().inner() {
                            let char_faith = ch.get_faith().as_ref().unwrap().clone();
                            let ch_faith = char_faith.get_internal();
                            let char_culture = ch.get_culture().as_ref().unwrap().clone();
                            let ch_culture = char_culture.get_internal();
                            if event == USURPED_STR || event.starts_with(CONQUERED_START_STR) {
                                let year: i16 = entry.0.year();
                                if ch_faith.get_id() != faith {
                                    events.push((
                                        year,
                                        char.clone(),
                                        title.clone(),
                                        GameString::from("faith"),
                                        RealmDifference::Faith(char_faith.clone()),
                                    ));
                                    faith = ch_faith.get_id();
                                } else if ch_culture.get_id() != culture {
                                    events.push((
                                        year,
                                        char.clone(),
                                        title.clone(),
                                        GameString::from("people"),
                                        RealmDifference::Culture(char_culture.clone()),
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
                }
            }
        }
        events.sort_by(|a, b| a.0.cmp(&b.0));
        return Timeline::new(lifespans, self.current_date.unwrap().year(), events);
    }
}

impl Localizable for GameState {
    fn localize<L: Localize<GameString>>(
        &mut self,
        localization: &mut L,
    ) -> Result<(), LocalizationError> {
        for character in self.characters.values_mut() {
            character.localize(localization)?;
        }
        for title in &mut self.titles.values_mut() {
            title.localize(localization)?;
            if let Some(internal) = title.get_internal_mut().inner_mut() {
                if let Some(assoc) = self.county_data.get_mut(internal.get_key().as_ref()) {
                    if let Title::County { faith, culture, .. } = internal {
                        *faith = Some(assoc.0.clone());
                        *culture = Some(assoc.1.clone());
                    }
                }
            }
        }
        for faith in &mut self.faiths.values_mut() {
            faith.localize(localization)?;
        }
        for culture in &mut self.cultures.values_mut() {
            culture.localize(localization)?;
        }
        for house in &mut self.houses.values_mut() {
            house.localize(localization)?;
        }
        for dynasty in &mut self.dynasties.values_mut() {
            dynasty.localize(localization)?;
        }
        for memory in &mut self.memories.values_mut() {
            memory.localize(localization)?;
        }
        for artifact in &mut self.artifacts.values_mut() {
            artifact.localize(localization)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

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
