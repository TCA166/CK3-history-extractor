use std::path::Path;

use serde::Serialize;

use super::{
    super::{
        display::{Grapher, ProceduralPath, Renderable},
        game_data::{GameData, Localizable, LocalizationError, Localize},
        jinja_env::DYN_TEMPLATE_NAME,
        parser::{GameObjectMap, GameObjectMapping, GameState, ParsingError, SaveFileValue},
        types::{GameString, HashMap, Wrapper},
    },
    Character, EntityRef, FromGameObject, GameObjectDerived, GameObjectEntity, GameRef, House,
};

#[derive(Serialize)]
pub struct Dynasty {
    name: Option<GameString>,
    prestige_tot: f32,
    prestige: f32,
    perks: HashMap<GameString, u8>,
    leader: Option<GameRef<Character>>,
    houses: Vec<GameRef<House>>,
}

impl FromGameObject for Dynasty {
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let mut val = Self {
            // name can also be stored as key, where it's either a string, or an ID pointing to an object declared in game files 00_dynasties.txt
            name: base
                .get("name")
                .or(base.get("dynasty_name"))
                .or(base.get("localized_name"))
                .and_then(|n| n.as_string().ok()),
            prestige_tot: 0.0,
            prestige: 0.0,
            perks: HashMap::new(),
            leader: None,
            houses: Vec::new(),
        };
        if let Some(leader) = base.get("dynasty_head") {
            val.leader = Some(game_state.get_character(&leader.as_id()?).clone());
        }
        if let Some(perks_obj) = base.get("perk") {
            for p in perks_obj.as_object()?.as_array()? {
                let perk = p.as_string()?;
                //get the split perk by the second underscore
                let mut i: u8 = 0;
                let mut key: Option<&str> = None;
                let mut level: u8 = 0;
                for el in perk.rsplitn(2, '_') {
                    if i == 0 {
                        level = el.parse::<u8>().unwrap();
                    } else {
                        key = Some(el);
                    }
                    i += 1;
                }
                if let Some(key) = key {
                    let key = GameString::from(key);
                    if *val.perks.entry(key.clone()).or_default() < level {
                        val.perks.insert(key, level);
                    }
                }
            }
        }
        if let Some(currency) = base.get("prestige") {
            let o = currency.as_object()?.as_map()?;
            if let Some(acc) = o.get("accumulated") {
                if let SaveFileValue::Object(o) = acc {
                    val.prestige_tot = o.as_map()?.get_real("value")? as f32;
                } else {
                    val.prestige_tot = acc.as_real()? as f32;
                }
            }
            if let Some(c) = o.get("currency") {
                if let SaveFileValue::Object(o) = c {
                    val.prestige = o.as_map()?.get_real("value")? as f32;
                } else {
                    val.prestige = c.as_real()? as f32;
                }
            }
        }
        return Ok(val);
    }

    fn finalize(&mut self, _reference: &GameRef<Self>) {
        self.houses.sort_by(|a, b| {
            a.get_internal()
                .inner()
                .unwrap()
                .get_found_date()
                .cmp(&b.get_internal().inner().unwrap().get_found_date())
        });
        // instead of resolving game files we can just get the name from the first house
        if self.name.is_none() {
            self.name = Some(
                self.houses
                    .first()
                    .unwrap()
                    .clone()
                    .get_internal()
                    .inner()
                    .unwrap()
                    .get_name()
                    .clone(),
            );
        }
    }
}

impl Dynasty {
    pub fn register_house(&mut self, house: GameRef<House>) {
        self.houses.push(house);
    }

    pub fn get_founder(&self) -> GameRef<Character> {
        self.houses
            .first()
            .unwrap()
            .clone()
            .get_internal()
            .inner()
            .unwrap()
            .get_founder()
    }

    pub fn get_leader(&self) -> Option<GameRef<Character>> {
        self.leader.clone()
    }
}

impl GameObjectDerived for Dynasty {
    fn get_name(&self) -> GameString {
        self.name.as_ref().unwrap().clone()
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
        if let Some(leader) = &self.leader {
            collection.extend([E::from(leader.clone().into())]);
        }
        for house in self.houses.iter() {
            collection.extend([E::from(house.clone().into())]);
        }
    }
}

impl ProceduralPath for Dynasty {
    fn get_subdir() -> &'static str {
        "dynasties"
    }
}

impl Renderable for GameObjectEntity<Dynasty> {
    fn get_template() -> &'static str {
        DYN_TEMPLATE_NAME
    }

    fn render(&self, path: &Path, _: &GameState, grapher: Option<&Grapher>, _: &GameData) {
        if let Some(grapher) = grapher {
            if let Some(dynasty) = self.inner() {
                let mut buf = path.join(Dynasty::get_subdir());
                buf.push(self.id.to_string() + ".svg");
                grapher.create_dynasty_graph(dynasty, &buf);
            }
        }
    }
}

impl Localizable for Dynasty {
    fn localize(&mut self, localization: &GameData) -> Result<(), LocalizationError> {
        if let Some(name) = &self.name {
            self.name = Some(localization.localize(name)?);
        }
        let drained_perks: Vec<_> = self.perks.drain().collect();
        for (perk, level) in drained_perks {
            self.perks.insert(
                localization.localize(perk.to_string() + "_track_name")?,
                level,
            );
        }
        Ok(())
    }
}
