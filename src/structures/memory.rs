use jomini::common::Date;
use serde::Serialize;

use super::{
    super::{
        game_data::{GameData, Localizable, LocalizationError, Localize},
        parser::{GameObjectMap, GameObjectMapping, GameState, ParsingError, SaveFileValue},
        types::{GameId, GameString, HashMap, Wrapper},
    },
    Character, EntityRef, FromGameObject, GameObjectDerived, GameRef,
};

#[derive(Serialize, Clone, Debug)]
enum MemoryVariable {
    Id(GameId),
    String(GameString),
    Bool(bool),
    None,
}

impl From<&GameObjectMap> for MemoryVariable {
    fn from(value: &GameObjectMap) -> Self {
        let tp = value.get_string("type").unwrap();
        match tp.as_ref() {
            "value" => MemoryVariable::None,
            "boolean" => MemoryVariable::Bool(value.get_integer("identity").unwrap() != 0),
            "trait" => MemoryVariable::String(value.get_string("key").unwrap()),
            "flag" => {
                if let Some(v) = value.get("flag") {
                    match v {
                        SaveFileValue::Integer(int) => MemoryVariable::Bool(*int != 0),
                        SaveFileValue::Boolean(b) => MemoryVariable::Bool(*b),
                        SaveFileValue::String(s) => MemoryVariable::String(s.clone()),
                        _ => unimplemented!("Unsupported type for flag: {:?}", v),
                    }
                } else {
                    MemoryVariable::None
                }
            }
            _ => value
                .get_game_id("identity")
                .and_then(|id| Ok(MemoryVariable::Id(id)))
                .unwrap_or(MemoryVariable::None),
        }
    }
}

/// A struct representing a memory in the game
#[derive(Serialize)]
pub struct Memory {
    date: Date,
    r#type: GameString,
    participants: HashMap<String, GameRef<Character>>,
    variables: HashMap<GameString, MemoryVariable>,
}

impl FromGameObject for Memory {
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let mut val = Self {
            date: base.get_date("creation_date")?,
            r#type: base.get_string("type")?,
            participants: HashMap::new(),
            variables: HashMap::new(),
        };
        if let Some(participants_node) = base.get("participants") {
            for part in participants_node.as_object()?.as_map()? {
                val.participants.insert(
                    part.0.clone(),
                    game_state.get_character(&part.1.as_id()?).clone(),
                );
            }
        }
        if let Some(variables_node) = base.get("variables") {
            let data_node = variables_node
                .as_object()?
                .as_map()?
                .get_object("data")?
                .as_array()?;
            for variable in data_node {
                let variable = variable.as_object()?.as_map()?;
                let key = variable.get_string("flag")?;
                val.variables.insert(
                    key.clone(),
                    MemoryVariable::from(variable.get_object("data")?.as_map()?),
                );
            }
        }
        Ok(val)
    }
}

impl GameObjectDerived for Memory {
    fn get_name(&self) -> GameString {
        self.r#type.clone()
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
        for part in self.participants.iter() {
            collection.extend([E::from(part.1.clone().into())]);
        }
    }
}

impl Localizable for Memory {
    fn localize(&mut self, localization: &GameData) -> Result<(), LocalizationError> {
        self.r#type = localization.localize_query(&self.r#type, |stack| {
            match stack.len() {
                4 => {
                    if stack[1].0 == "Var" {
                        if stack[2].0 == "GetProvince" {
                            if let Some(province_id) = self.variables.get(stack[1].1[0].as_str()) {
                                if let MemoryVariable::Id(id) = province_id {
                                    if let Some(province_name) = localization.lookup_title(id) {
                                        if let Ok(province_name) =
                                            localization.localize(province_name)
                                        {
                                            return Some(province_name);
                                        }
                                    }
                                }
                            } else {
                                return Some(stack[1].1[0].clone().into());
                            }
                        } else if stack[2].0 == "Trait" {
                            if let Some(trait_name) = self.variables.get(stack[1].1[0].as_str()) {
                                if let MemoryVariable::String(trait_name) = trait_name {
                                    return Some(trait_name.clone());
                                }
                            }
                        }
                    }
                }
                3 => {
                    if stack[1].0 == "Var" {
                        if let Some(var_name) = self.variables.get(stack[1].1[0].as_str()) {
                            if let MemoryVariable::String(var_name) = var_name {
                                return Some(var_name.clone());
                            }
                        }
                    }
                }
                2 => {
                    if stack[0].0 == "owner" {
                        if stack[1].0 == "GetName" || stack[1].0 == "GetTitledFirstName" {
                            return Some("".into());
                        } else if stack[1].0 == "GetHerHis" {
                            return Some("my".into());
                        } else if stack[1].1[0] == "RelationToMeShort"
                            || stack[1].1[1] == "RelationToMeShort"
                        {
                            if let Some(guy) =
                                self.participants.get(stack[1].1[1].as_str().trim_start())
                            {
                                return Some(guy.get_internal().inner().unwrap().get_name());
                            }
                        }
                    } else if stack[0].0 == "predecessor" {
                        if stack[1].0 == "GetHerHis" {
                            return Some("their".into());
                        } else if stack[1].0 == "GetHerHim" {
                            return Some("them".into());
                        }
                    } else if let Some(part) = self.participants.get(stack[0].0.as_str()) {
                        if stack[1].0 == "GetName" || stack[1].0 == "GetTitledFirstName" {
                            return Some(part.get_internal().inner().unwrap().get_name());
                        } else if stack[1].0 == "GetHerHis" || stack[1].0 == "GetNamePossessive" {
                            if part.get_internal().inner().unwrap().get_female() {
                                return Some("Her".into());
                            } else {
                                return Some("His".into());
                            }
                        }
                    } else if stack[1].0 == "GetName" || stack[1].0 == "GetTitledFirstName" {
                        return Some(stack[0].1[0].clone().into());
                    }
                    if stack[1].0 == "Custom" {
                        if stack[1].1[0] == "KnightCultureNoTooltip" {
                            return Some("Knight".into());
                        }
                    }
                }
                _ => {}
            }
            None
        })?;
        Ok(())
    }
}

impl Serialize for GameRef<Memory> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get_internal().serialize(serializer)
    }
}
