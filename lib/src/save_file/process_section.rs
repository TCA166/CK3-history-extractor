use std::{collections::HashMap, io::Read};

use derive_more::From;

use super::{
    game_state::GameState,
    parser::{
        types::GameId, BinarySection, BinarySectionReader, GameObjectMapping, KeyError,
        ParsingError, SaveFileObject, SaveFileSection, SaveFileValue, SectionError, TextSection,
        TextSectionReader,
    },
    structures::{FromGameObject, Player},
};

/// Abstract section reader. Holds a reference to the contents, and
/// returns abstract [Section]s.
#[derive(From)]
pub enum SectionReader<'resolver, R: Read> {
    Text(TextSectionReader<R>),
    Binary(BinarySectionReader<'resolver, R>),
}

impl<'resolver, R: Read> SectionReader<'resolver, R> {
    /// Returns the next section, or None if there are no more sections.
    pub fn next(&mut self) -> Option<Result<Section<'_, 'resolver, R>, ParsingError>> {
        match self {
            SectionReader::Binary(b) => b.next().and_then(|res| {
                Some(match res {
                    Ok(val) => Ok(val.into()),
                    Err(val) => Err(val.into()),
                })
            }),
            SectionReader::Text(t) => t.next().and_then(|res| {
                Some(match res {
                    Ok(val) => Ok(val.into()),
                    Err(val) => Err(val.into()),
                })
            }),
        }
    }

    /// Processes all sections in the reader.
    /// Internally, this calls [Section::process_section] on each section.
    /// Use this when you just want to populate [GameState], and you don't
    /// care which sections get skipped.
    pub fn process_sections(
        mut self,
        game_state: &mut GameState,
        players: &mut Vec<Player>,
    ) -> Result<(), ParsingError> {
        while let Some(res) = self.next() {
            let section = res?;
            section.process_section(game_state, players)?;
        }
        Ok(())
    }
}

/// Abstract section, which can be either text or binary.
/// All of the functionality is encapsulated in the [SaveFileSection] trait.
#[derive(From)]
pub enum Section<'tape, 'resolver, R: Read> {
    Text(TextSection<'tape, R>),
    Binary(BinarySection<'tape, 'resolver, R>),
}

impl<R: Read> SaveFileSection for Section<'_, '_, R> {
    fn get_name(&self) -> &str {
        match self {
            Section::Text(section) => section.get_name(),
            Section::Binary(section) => section.get_name(),
        }
    }

    fn skip(self) -> Result<(), SectionError> {
        match self {
            Section::Text(section) => section.skip(),
            Section::Binary(section) => section.skip(),
        }
    }

    fn parse(self) -> Result<SaveFileObject, SectionError> {
        match self {
            Section::Text(section) => section.parse(),
            Section::Binary(section) => section.parse(),
        }
    }
}

impl<R: Read> Section<'_, '_, R> {
    /// Conditionally parses the section based on its name, and uses the
    /// contained data to populate the [GameState] and a list of [Player]s.
    pub fn process_section(
        self,
        game_state: &mut GameState,
        players: &mut Vec<Player>,
    ) -> Result<(), ParsingError> {
        match self.get_name() {
            "meta_data" => {
                let parsed = self.parse()?;
                let map = parsed.as_map()?;
                game_state
                    .set_current_date(map.get_date("meta_date")?, map.get_date("meta_real_date")?);
            }
            //the order is kept consistent with the order in the save file
            "traits_lookup" => {
                let lookup: Result<Vec<_>, _> = self
                    .parse()?
                    .as_array()?
                    .into_iter()
                    .map(|x| x.as_string())
                    .collect();
                game_state.add_lookup(lookup?);
            }
            "landed_titles" => {
                let parsed = self.parse()?;
                let map = parsed.as_map()?;
                for (key, v) in map.get_object("landed_titles")?.as_map()? {
                    if let SaveFileValue::Object(o) = v {
                        game_state.add_title(&key.parse::<GameId>()?, o.as_map()?)?;
                    }
                }
            }
            "county_manager" => {
                let parsed = self.parse()?;
                let map = parsed.as_map()?;
                // we create an association between the county key and the faith and culture of the county
                // this is so that we can easily add the faith and culture to the title, so O(n) instead of O(n^2)
                let mut key_assoc = HashMap::default();
                for (key, p) in map.get_object("counties")?.as_map()? {
                    let p = p.as_object()?.as_map()?;
                    let faith = game_state.get_faith(&p.get_game_id("faith")?);
                    let culture = game_state.get_culture(&p.get_game_id("culture")?);
                    key_assoc.insert(key.to_owned(), (faith, culture));
                }
                game_state.add_county_data(key_assoc);
            }
            "dynasties" => {
                let m = self.parse()?;
                let m = m.as_map()?;
                for (key, house) in m.get("dynasty_house").unwrap().as_object()?.as_map()? {
                    if let SaveFileValue::Object(o) = house {
                        game_state.add_house(&key.parse::<GameId>()?, o.as_map()?)?;
                    }
                }
                for (key, dynasty) in m.get("dynasties").unwrap().as_object()?.as_map()? {
                    if let SaveFileValue::Object(o) = dynasty {
                        game_state.add_dynasty(&key.parse::<GameId>()?, o.as_map()?)?;
                    }
                }
            }
            "character_lookup" => {
                let parsed = self.parse()?;
                let mut transform = HashMap::new();
                for (key, val) in parsed.as_map()? {
                    if let Ok(key) = key.parse::<GameId>() {
                        transform.insert(key, val.as_id()?);
                    }
                }
                game_state.add_character_transform(transform);
            }
            "living" => {
                for (key, l) in self.parse()?.as_map()? {
                    match l {
                        SaveFileValue::Object(o) => {
                            game_state.add_character(&key.parse::<GameId>()?, o.as_map()?)?;
                        }
                        _ => {
                            continue;
                        }
                    }
                }
            }
            "dead_unprunable" => {
                for (key, d) in self.parse()?.as_map()? {
                    if let SaveFileValue::Object(o) = d {
                        game_state.add_character(&key.parse::<GameId>()?, o.as_map()?)?;
                    }
                }
            }
            "characters" => {
                if let Some(dead_prunable) = self.parse()?.as_map()?.get("dead_prunable") {
                    for (key, d) in dead_prunable.as_object()?.as_map()? {
                        match d {
                            SaveFileValue::Object(o) => {
                                game_state.add_character(&key.parse::<GameId>()?, o.as_map()?)?;
                            }
                            _ => {
                                continue;
                            }
                        }
                    }
                }
            }
            "vassal_contracts" => {
                let r = self.parse()?;
                let map = r.as_map()?;
                // if version <= 1.12 then the key is active, otherwise it is database, why paradox?
                for (key, contract) in map
                    .get("database")
                    .or(map.get("active"))
                    .ok_or_else(|| KeyError::MissingKey("database".to_string(), map.clone()))?
                    .as_object()?
                    .as_map()?
                {
                    if let SaveFileValue::Object(val) = contract {
                        let val = val.as_map()?;
                        game_state.add_contract(
                            &key.parse::<GameId>().unwrap(),
                            &val.get_game_id("vassal")?,
                        )
                    }
                }
            }
            "religion" => {
                let parsed = self.parse()?;
                let map = parsed.as_map()?;
                for (key, f) in map.get_object("faiths")?.as_map()? {
                    game_state.add_faith(&key.parse::<GameId>()?, f.as_object()?.as_map()?)?;
                }
            }
            "culture_manager" => {
                let parsed = self.parse()?;
                let map = parsed.as_map()?;
                let cultures = map.get_object("cultures")?.as_map()?;
                for (key, c) in cultures {
                    game_state.add_culture(&key.parse::<GameId>()?, c.as_object()?.as_map()?)?;
                }
            }
            "character_memory_manager" => {
                let parsed = self.parse()?;
                let map = parsed.as_map()?;
                for (key, d) in map.get_object("database")?.as_map()? {
                    if let SaveFileValue::Object(o) = d {
                        game_state.add_memory(&key.parse::<GameId>()?, o.as_map()?)?;
                    }
                }
            }
            "played_character" => {
                let p = Player::from_game_object(self.parse()?.as_map()?, game_state)?;
                players.push(p);
            }
            "artifacts" => {
                let parsed = self.parse()?;
                let map = parsed.as_map()?;
                for (key, a) in map.get_object("artifacts")?.as_map()?.into_iter() {
                    if let SaveFileValue::Object(o) = a {
                        game_state.add_artifact(&key.parse::<GameId>()?, o.as_map()?)?;
                    }
                }
            }
            _ => {
                self.skip()?;
            }
        }
        return Ok(());
    }
}
