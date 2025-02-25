use std::path::Path;

use jomini::common::Date;
use serde::Serialize;

use super::{
    super::{
        display::{Grapher, ProceduralPath, Renderable},
        game_data::{GameData, Localizable, LocalizationError, Localize},
        jinja_env::DYN_TEMPLATE_NAME,
        parser::{
            GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError, SaveFileValue,
        },
        types::{HashMap, Wrapper, WrapperMut},
    },
    Character, EntityRef, FromGameObject, GameObjectDerived, GameObjectEntity, GameRef,
};

// TODO figure out how to handle houses and dynasties

#[derive(Serialize, Debug)]
pub struct Dynasty {
    parent: Option<GameRef<Dynasty>>,
    name: Option<GameString>,
    members: u32,
    member_list: Vec<GameRef<Character>>,
    houses: u32,
    prestige_tot: f32,
    prestige: f32,
    perks: HashMap<GameString, u8>,
    leaders: Vec<GameRef<Character>>,
    found_date: Option<Date>,
    motto: Option<(GameString, HashMap<i64, GameString>)>,
}

impl Dynasty {
    pub fn get_leader(&self) -> GameRef<Character> {
        if self.leaders.is_empty() {
            return self.member_list.last().unwrap().clone();
        }
        self.leaders.last().unwrap().clone()
    }

    /// Registers a new house in the dynasty
    pub fn register_house(&mut self) {
        self.houses += 1;
    }

    /// Registers a new member in the dynasty
    pub fn register_member(&mut self, member: GameRef<Character>) {
        self.members += 1;
        self.member_list.push(member.clone());
        if let Some(parent) = &self.parent {
            if let Ok(mut p) = parent.try_get_internal_mut() {
                if let Some(p) = p.inner_mut() {
                    p.register_member(member);
                }
            }
        }
    }

    /// Gets the founder of the dynasty
    pub fn get_founder(&self) -> GameRef<Character> {
        if self.leaders.is_empty() {
            return self.member_list.first().unwrap().clone();
        }
        self.leaders.first().unwrap().clone()
    }
}

impl GameObjectEntity<Dynasty> {
    /// Checks if the dynasty is the same as another dynasty
    pub fn is_same_dynasty(&self, other: &Self) -> bool {
        let id = if let Some(parent) = self.entity.as_ref().and_then(|x| x.parent.clone()) {
            parent.get_internal().id
        } else {
            self.id
        };
        if let Some(other_parent) = &other.entity.as_ref().and_then(|x| x.parent.clone()) {
            return id == other_parent.get_internal().id;
        } else {
            return id == other.id;
        }
    }
}

impl FromGameObject for Dynasty {
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        //NOTE: dynasties can have their main house have the same id
        let mut val = Self {
            name: base
                .get("name")
                .or(base.get("localized_name"))
                .map(|n| n.as_string())
                .transpose()?,
            found_date: base.get("found_date").map(|n| n.as_date()).transpose()?,
            motto: None,
            parent: base
                .get("dynasty")
                .map(|n| n.as_id().and_then(|id| Ok(game_state.get_dynasty(&id))))
                .transpose()?,
            members: 0,
            member_list: Vec::new(),
            houses: 0,
            prestige_tot: 0.0,
            prestige: 0.0,
            perks: HashMap::new(),
            leaders: Vec::new(),
        };
        if let Some(motto_node) = base.get("motto") {
            if let SaveFileValue::Object(obj) = motto_node {
                let o = obj.as_map()?;
                let mut vars = HashMap::new();
                for v in o.get_object("variables")?.as_array()? {
                    let pair = v.as_object()?.as_map()?;
                    vars.insert(pair.get_integer("key")?, pair.get_string("value")?);
                }
                val.motto = Some((o.get_string("key")?.clone(), vars));
            } else {
                val.motto = Some((motto_node.as_string()?, HashMap::default()));
            }
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
        if let Some(leaders_obj) = base.get("historical") {
            if !val.leaders.is_empty() {
                val.leaders.clear();
            }
            for l in leaders_obj.as_object()?.as_array()? {
                val.leaders
                    .push(game_state.get_character(&l.as_id()?).clone());
            }
        } else if val.leaders.is_empty() {
            if let Some(current) = base.get("dynasty_head").or(base.get("head_of_house")) {
                val.leaders
                    .push(game_state.get_character(&current.as_id()?));
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
}

impl GameObjectDerived for Dynasty {
    fn get_name(&self) -> GameString {
        self.name
            .clone()
            .or(self
                .parent
                .as_ref()
                .and_then(|x| x.get_internal().inner().and_then(|x| x.name.clone())))
            .unwrap_or("Unknown".into())
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
        for leader in self.leaders.iter() {
            collection.extend([E::from(leader.clone().into())]);
        }
        if let Some(parent) = &self.parent {
            collection.extend([E::from(parent.clone().into())]);
        }
    }

    fn finalize(&mut self, reference: &GameRef<Dynasty>) {
        if let Some(parent) = &self.parent {
            if !reference
                .try_get_internal()
                .is_ok_and(|this| this.id != parent.get_internal().id)
            {
                self.parent = None;
            } else {
                // MAYBE this is bad? I don't know
                if let Ok(mut p) = parent.try_get_internal_mut() {
                    if let Some(p) = p.inner_mut() {
                        p.register_house();
                    }
                }
            }
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
                buf.push(format!("{}.svg", self.id));
                grapher.create_dynasty_graph(dynasty, &buf);
            }
        }
    }
}

impl Localizable for Dynasty {
    fn localize<L: Localize<GameString>>(
        &mut self,
        localization: &mut L,
    ) -> Result<(), LocalizationError> {
        if let Some(name) = &self.name {
            self.name = Some(localization.localize(name)?);
        } else {
            return Ok(());
        }
        let drained_perks: Vec<_> = self.perks.drain().collect();
        for (perk, level) in drained_perks {
            self.perks.insert(
                localization.localize(perk.to_string() + "_track_name")?,
                level,
            );
        }
        if let Some((motto, variables)) = &mut self.motto {
            *motto = localization.localize_query(&motto, |stack| {
                if stack.len() == 1 {
                    if let Ok(k) = stack[0].0.parse::<i64>() {
                        if let Some(v) = variables.get(&k) {
                            return Some(v.clone());
                        }
                    }
                } else if stack.len() == 2 {
                    if stack[0].0 == "CHARACTER" && stack.len() >= 2 {
                        if let Some(leader) =
                            self.leaders.first().unwrap().clone().get_internal().inner()
                        {
                            if stack[1].0 == "Custom" && stack[1].1.len() == 1 {
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
                            } else if stack[1].0 == "GetFirstName" {
                                return Some(leader.get_name().clone());
                            }
                        } else {
                            return Some("Unknown".into());
                        }
                    }
                } else if stack[2].0 == "GetBaseNameNoTooltip" {
                    return Some(self.name.as_ref().unwrap().clone());
                }
                None
            })?;
        }
        Ok(())
    }
}
