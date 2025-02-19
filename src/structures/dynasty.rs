use std::path::Path;

use jomini::common::Date;
use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        display::{Cullable, Grapher, Renderable, RenderableType},
        game_data::{GameData, Localizable, LocalizationError, Localize},
        jinja_env::DYN_TEMPLATE_NAME,
        parser::{
            GameObjectMap, GameObjectMapping, GameState, GameString, ParsingError, SaveFileValue,
        },
        types::{HashMap, Wrapper, WrapperMut},
    },
    into_ref_array, Character, DerivedRef, DummyInit, GameId, GameObjectDerived, Shared,
};

pub struct Dynasty {
    id: GameId,
    parent: Option<Shared<Dynasty>>,
    name: Option<GameString>,
    members: u32,
    member_list: Vec<Shared<Character>>,
    houses: u32,
    prestige_tot: f32,
    prestige: f32,
    perks: HashMap<GameString, u8>,
    leaders: Vec<Shared<Character>>,
    found_date: Option<Date>,
    motto: Option<(GameString, Vec<GameString>)>,
    depth: usize,
}

impl Dynasty {
    pub fn get_leader(&self) -> Shared<Character> {
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
    pub fn register_member(&mut self, member: Shared<Character>) {
        self.members += 1;
        self.member_list.push(member.clone());
        if let Some(parent) = &self.parent {
            if let Ok(mut p) = parent.try_get_internal_mut() {
                p.register_member(member);
            }
        }
    }

    /// Gets the founder of the dynasty
    pub fn get_founder(&self) -> Shared<Character> {
        if self.leaders.is_empty() {
            return self.member_list.first().unwrap().clone();
        }
        self.leaders.first().unwrap().clone()
    }

    /// Checks if the dynasty is the same as another dynasty
    pub fn is_same_dynasty(&self, other: &Dynasty) -> bool {
        let id = if let Some(parent) = &self.parent {
            parent.get_internal().id
        } else {
            self.id
        };
        if let Some(other_parent) = &other.parent {
            return id == other_parent.get_internal().id;
        } else {
            return id == other.id;
        }
    }
}

impl DummyInit for Dynasty {
    fn dummy(id: GameId) -> Self {
        Dynasty {
            name: None,
            parent: None,
            members: 0,
            houses: 0,
            prestige_tot: 0.0,
            prestige: 0.0,
            perks: HashMap::new(),
            leaders: Vec::new(),
            found_date: None,
            id: id,
            depth: 0,
            motto: None,
            member_list: Vec::new(),
        }
    }

    fn init(
        &mut self,
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<(), ParsingError> {
        //NOTE: dynasties can have their main house have the same id
        if let Some(perks_obj) = base.get("perk") {
            for p in perks_obj.as_object()?.as_array()? {
                let perk = p.as_string()?;
                //get the split perk by the second underscore
                let mut i: u8 = 0;
                let mut key: Option<&str> = None;
                let mut val: u8 = 0;
                for el in perk.rsplitn(2, '_') {
                    if i == 0 {
                        val = el.parse::<u8>().unwrap();
                    } else {
                        key = Some(el);
                    }
                    i += 1;
                }
                if let Some(key) = key {
                    let key = GameString::from(key);
                    if *self.perks.entry(key.clone()).or_default() < val {
                        self.perks.insert(key, val);
                    }
                }
            }
        }
        if let Some(leaders_obj) = base.get("historical") {
            if !self.leaders.is_empty() {
                self.leaders.clear();
            }
            for l in leaders_obj.as_object()?.as_array()? {
                self.leaders
                    .push(game_state.get_character(&l.as_id()?).clone());
            }
        } else if self.leaders.is_empty() {
            if let Some(current) = base.get("dynasty_head").or(base.get("head_of_house")) {
                self.leaders
                    .push(game_state.get_character(&current.as_id()?));
            }
        }
        if let Some(currency) = base.get("prestige") {
            let o = currency.as_object()?.as_map()?;
            if let Some(acc) = o.get("accumulated") {
                if let SaveFileValue::Object(o) = acc {
                    self.prestige_tot = o.as_map()?.get_real("value")? as f32;
                } else {
                    self.prestige_tot = acc.as_real()? as f32;
                }
            }
            if let Some(c) = o.get("currency") {
                if let SaveFileValue::Object(o) = c {
                    self.prestige = o.as_map()?.get_real("value")? as f32;
                } else {
                    self.prestige = c.as_real()? as f32;
                }
            }
        }
        if let Some(paret) = base.get("dynasty") {
            let paret = paret.as_id()?;
            if paret != self.id {
                // MAYBE this is bad? I don't know
                let p = game_state.get_dynasty(&paret).clone();
                if let Ok(mut p) = p.try_get_internal_mut() {
                    p.register_house();
                }
                self.parent = Some(p.clone());
            }
        }
        match base.get("name").or(base.get("localized_name")) {
            Some(name) => {
                self.name = Some(name.as_string()?);
            }
            None => {
                //this may happen for dynasties with a house with the same name
                if let Some(parent) = &self.parent {
                    if let Some(name) = &parent.get_internal().name {
                        self.name = Some(name.clone());
                    }
                }
            }
        }
        if let Some(date) = base.get("found_date") {
            self.found_date = Some(date.as_date()?);
        }
        if let Some(motto_node) = base.get("motto") {
            if let SaveFileValue::Object(obj) = motto_node {
                let o = obj.as_map()?;
                let mut vars = Vec::new();
                for v in o.get_object("variables")?.as_array()? {
                    let value = v.as_object()?.as_map()?.get_string("value")?;
                    vars.push(value.clone());
                }
                self.motto = Some((o.get_string("key")?.clone(), vars));
            } else {
                self.motto = Some((motto_node.as_string()?, Vec::new()));
            }
        }
        Ok(())
    }
}

impl GameObjectDerived for Dynasty {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        if let Some(name) = &self.name {
            return name.clone();
        } else if let Some(parent) = &self.parent {
            return parent.get_internal().get_name();
        } else {
            return GameString::from("Unknown");
        }
    }
}

impl Serialize for Dynasty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Dynasty", 11)?;
        state.serialize_field("id", &self.id)?;
        if let Some(parent) = &self.parent {
            let parent = DerivedRef::<Dynasty>::from(parent.clone());
            state.serialize_field("parent", &parent)?;
        }
        state.serialize_field("name", &self.name)?;
        state.serialize_field("members", &self.members)?;
        state.serialize_field("houses", &self.houses)?;
        state.serialize_field("prestige_tot", &self.prestige_tot)?;
        state.serialize_field("prestige", &self.prestige)?;
        if !self.perks.is_empty() {
            state.serialize_field("perks", &self.perks)?;
        }
        let leaders = into_ref_array(&self.leaders);
        state.serialize_field("leaders", &leaders)?;
        state.serialize_field("found_date", &self.found_date)?;
        if let Some(motto_raw) = &self.motto {
            state.serialize_field("motto", &motto_raw.0)?;
        }
        state.end()
    }
}

impl Renderable for Dynasty {
    fn get_template() -> &'static str {
        DYN_TEMPLATE_NAME
    }

    fn get_subdir() -> &'static str {
        "dynasties"
    }

    fn render(&self, path: &Path, _: &GameState, grapher: Option<&Grapher>, _: &GameData) {
        if let Some(grapher) = grapher {
            let mut buf = path.join(Self::get_subdir());
            buf.push(format!("{}.svg", self.id));
            grapher.create_dynasty_graph(self, &buf);
        }
    }

    fn append_ref(&self, stack: &mut Vec<RenderableType>) {
        for leader in self.leaders.iter() {
            stack.push(RenderableType::Character(leader.clone()));
        }
        if let Some(parent) = &self.parent {
            stack.push(RenderableType::Dynasty(parent.clone()));
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
        if let Some(motto) = &mut self.motto {
            motto.0 = localization.localize(&motto.0)?;
            for v in motto.1.iter_mut() {
                *v = localization.localize(&v)?;
            }
            // TODO localize the motto properly here
            /*
            ("motto_the_ancient_x_is_ours", ["motto_family"])
            ("The Ancient  is Ours", ["Family"])
            ("motto_unique_pool", ["motto_more_than_silver"])
            ("", ["Trust From a  is Worth More than Silver"])
            ("motto_through_x_mind_y", ["motto_an_honorable", "motto_respect"])
            ("Through  Mind, ", ["an Honorable", "Respect"])
            ("motto_x_is_y", ["motto_valor", "motto_boldness"])
            (" is ", ["Valor", "Boldness"])
            ("motto_single_noun", ["motto_labour"])
            ("", ["Labor"])
                         */
        }
        Ok(())
    }
}

impl Cullable for Dynasty {
    fn set_depth(&mut self, depth: usize) {
        if depth <= self.depth {
            return;
        }
        self.depth = depth;
        let depth = depth - 1;
        for leader in self.leaders.iter() {
            if let Ok(mut o) = leader.try_get_internal_mut() {
                o.set_depth(depth);
            }
        }
        if let Some(parent) = &self.parent {
            if let Ok(mut o) = parent.try_get_internal_mut() {
                o.set_depth(depth);
            }
        }
    }

    fn get_depth(&self) -> usize {
        self.depth
    }
}
