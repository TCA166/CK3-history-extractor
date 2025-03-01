use jomini::common::Date;

use serde::Serialize;

use super::{
    super::{
        display::{ProceduralPath, Renderable},
        game_data::{Localizable, LocalizationError, Localize},
        jinja_env::HOUSE_TEMPLATE_NAME,
        parser::{
            GameObjectMap, GameObjectMapping, GameState, KeyError, ParsingError, SaveFileValue,
            SaveObjectError,
        },
        types::{GameString, HashMap, Wrapper, WrapperMut},
    },
    Character, Culture, Dynasty, EntityRef, Faith, FromGameObject, GameObjectDerived,
    GameObjectEntity, GameRef,
};

#[derive(Serialize)]
pub struct House {
    name: GameString,
    parent: GameRef<Dynasty>,
    leaders: Vec<GameRef<Character>>,
    motto: Option<(GameString, HashMap<i64, GameString>)>,
    found_date: Option<Date>,
}

fn get_house_name(base: &GameObjectMap) -> Result<GameString, ParsingError> {
    if let Some(name) = base.get("name").or(base.get("localized_name")) {
        Ok(name.as_string()?)
    } else {
        match base.get_err("key")? {
            SaveFileValue::Integer(id) => Ok(id.to_string().into()),
            SaveFileValue::String(name) => Ok(name.clone()),
            _ => Err(ParsingError::StructureError(SaveObjectError::KeyError(
                KeyError::MissingKey("house_name or localized_name".to_string(), base.to_owned()),
            ))),
        }
    }
}

impl FromGameObject for House {
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let mut this = Self {
            name: get_house_name(base)?,
            parent: game_state
                .get_dynasty(&base.get_game_id("dynasty")?)
                .clone(),
            found_date: base.get("found_date").map(|n| n.as_date()).transpose()?,
            leaders: Vec::new(),
            motto: None,
        };
        if let Some(motto_node) = base.get("motto") {
            if let SaveFileValue::Object(obj) = motto_node {
                let o = obj.as_map()?;
                let mut vars = HashMap::new();
                for v in o.get_object("variables")?.as_array()? {
                    let pair = v.as_object()?.as_map()?;
                    vars.insert(pair.get_integer("key")?, pair.get_string("value")?);
                }
                this.motto = Some((o.get_string("key")?.clone(), vars));
            } else {
                this.motto = Some((motto_node.as_string()?, HashMap::default()));
            }
        }
        if let Some(leaders_obj) = base.get("historical") {
            for l in leaders_obj.as_object()?.as_array()? {
                this.leaders
                    .push(game_state.get_character(&l.as_id()?).clone());
            }
        }
        Ok(this)
    }

    fn finalize(&mut self, reference: &GameRef<Self>) {
        self.parent
            .get_internal_mut()
            .inner_mut()
            .unwrap()
            .register_house(reference.clone());
    }
}

impl GameObjectDerived for House {
    fn get_name(&self) -> GameString {
        self.name.clone()
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
        for leader in self.leaders.iter() {
            collection.extend([E::from(leader.clone().into())]);
        }
        collection.extend([E::from(self.parent.clone().into())]);
    }
}

impl Localizable for House {
    fn localize<L: Localize<GameString>>(
        &mut self,
        localization: &mut L,
    ) -> Result<(), LocalizationError> {
        self.name = localization.localize(&self.name)?;
        let query = |stack: &Vec<(String, Vec<String>)>| {
            match stack.len() {
                2 => {
                    match stack[0].0.as_str() {
                        "CHARACTER" => {
                            if let Some(leader) = self
                                .leaders
                                .iter()
                                .find(|l| l.get_internal().inner().is_some())
                            {
                                let leader = leader.get_internal();
                                let leader = leader.inner().unwrap();
                                match stack[1].0.as_str() {
                                    "Custom" => {
                                        if stack[1].1[0] == "GetAppropriateGodname" {
                                            // TODO localize the godname properly here
                                            return Some("God".into());
                                        } else if stack[1].1[0] == "QueenKing" {
                                            if leader.get_female() {
                                                return Some("Queen".into());
                                            } else {
                                                return Some("King".into());
                                            }
                                        } else if stack[1].1[0] == "GetDaughterSon" {
                                            if leader.get_female() {
                                                return Some("Daughter".into());
                                            } else {
                                                return Some("Son".into());
                                            }
                                        }
                                    }
                                    "GetFirstName" => {
                                        return Some(leader.get_name().clone());
                                    }
                                    "GetSheHe" => {
                                        if leader.get_female() {
                                            return Some("She".into());
                                        } else {
                                            return Some("He".into());
                                        }
                                    }
                                    "GetWomenMen" => {
                                        if leader.get_female() {
                                            return Some("Women".into());
                                        } else {
                                            return Some("Men".into());
                                        }
                                    }
                                    _ => {}
                                }
                            } else {
                                // TODO this can happen with uninitialized characters
                                return Some("Unknown".into());
                            }
                        }
                        _ => {}
                    }
                }
                3 => {
                    if stack[2].0 == "GetBaseNameNoTooltip" {
                        return Some(self.name.clone());
                    }
                }
                _ => {}
            };
            None
        };
        if let Some((motto, variables)) = &mut self.motto {
            for (_, v) in variables.iter_mut() {
                *v = localization.localize_query(&v, query)?;
            }
            *motto = localization.localize_query(&motto, |stack| {
                match stack.len() {
                    1 => {
                        if let Ok(k) = stack[0].0.parse::<i64>() {
                            if let Some(v) = variables.get(&k) {
                                return Some(v.clone());
                            }
                        }
                    }
                    _ => {
                        return query(stack);
                    }
                }
                None
            })?;
        }
        Ok(())
    }
}

impl House {
    pub fn get_faith(&self) -> GameRef<Faith> {
        for leader in self.leaders.iter().rev() {
            if let Ok(faith) = leader.try_get_internal() {
                if let Some(faith) = faith.inner() {
                    if let Some(faith) = faith.get_faith() {
                        return faith;
                    }
                }
            }
        }
        unimplemented!()
    }

    pub fn get_culture(&self) -> GameRef<Culture> {
        for leader in self.leaders.iter().rev() {
            if let Ok(culture) = leader.try_get_internal() {
                if let Some(culture) = culture.inner() {
                    if let Some(culture) = culture.get_culture() {
                        return culture;
                    }
                }
            }
        }
        unimplemented!()
    }

    pub fn get_founder(&self) -> GameRef<Character> {
        if let Some(leader) = self.leaders.first() {
            leader.clone()
        } else {
            self.parent
                .get_internal()
                .inner()
                .unwrap()
                .get_leader()
                .unwrap()
        }
    }

    pub fn get_dynasty(&self) -> GameRef<Dynasty> {
        self.parent.clone()
    }

    pub fn get_found_date(&self) -> Option<Date> {
        self.found_date
    }
}

impl ProceduralPath for House {
    fn get_subdir() -> &'static str {
        "houses"
    }
}

impl Renderable for GameObjectEntity<House> {
    fn get_template() -> &'static str {
        HOUSE_TEMPLATE_NAME
    }
}
