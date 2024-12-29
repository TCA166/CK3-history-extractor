use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        display::{Cullable, GameMap, Grapher, Localizable, Localizer, Renderable, RenderableType},
        jinja_env::DYN_TEMPLATE_NAME,
        parser::{GameObjectMap, GameState, GameString, ParsingError, SaveFileValue},
        types::{Wrapper, WrapperMut},
    },
    serialize_array, Character, Culture, DerivedRef, DummyInit, Faith, GameId, GameObjectDerived,
    Shared,
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
    perks: Vec<(GameString, u8)>,
    leaders: Vec<Shared<Character>>,
    found_date: Option<GameString>,
    motto: Option<(GameString, Vec<GameString>)>,
    depth: usize,
}

impl Dynasty {
    /// Gets the faith of the dynasty.
    /// Really this is just the faith of the current house leader.
    pub fn get_faith(&self) -> Option<Shared<Faith>> {
        if self.leaders.is_empty() {
            return self.member_list.last().unwrap().get_internal().get_faith();
        }
        self.leaders.last().unwrap().get_internal().get_faith()
    }

    /// Gets the culture of the dynasty.
    /// Really this is just the culture of the current house leader.
    pub fn get_culture(&self) -> Option<Shared<Culture>> {
        if self.leaders.is_empty() {
            return self
                .member_list
                .last()
                .unwrap()
                .get_internal()
                .get_culture();
        }
        self.leaders.last().unwrap().get_internal().get_culture()
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
            perks: Vec::new(),
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
                if key.is_none() {
                    continue;
                }
                let mut added = false;
                // TODO why not hashmap?
                for perk in self.perks.iter_mut() {
                    if perk.0.as_str() == key.unwrap() {
                        if perk.1 < val {
                            perk.1 = val;
                        }
                        added = true;
                        break;
                    }
                }
                if added {
                    continue;
                }
                //if the perk is not found, add it
                self.perks
                    .push((GameString::wrap(key.unwrap().to_owned()), val));
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
            self.found_date = Some(date.as_string()?);
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
            return GameString::wrap("Unknown".to_owned());
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
            let parent = DerivedRef::<Dynasty>::from_derived(parent.clone());
            state.serialize_field("parent", &parent)?;
        }
        state.serialize_field("name", &self.name)?;
        state.serialize_field("members", &self.members)?;
        state.serialize_field("houses", &self.houses)?;
        state.serialize_field("prestige_tot", &self.prestige_tot)?;
        state.serialize_field("prestige", &self.prestige)?;
        state.serialize_field("perks", &self.perks)?;
        let leaders = serialize_array(&self.leaders);
        state.serialize_field("leaders", &leaders)?;
        state.serialize_field("found_date", &self.found_date)?;
        if let Some(motto_raw) = &self.motto {
            let motto = motto_raw.0.split(' ').collect::<Vec<&str>>();
            let var_len = motto_raw.1.len();
            let rebuilt = if var_len == 0 {
                motto_raw.0.clone()
            } else {
                let mut rebuilt = Vec::new();
                let mut j = 0;
                for part in motto {
                    if part.is_empty() || part == "," {
                        rebuilt.push(motto_raw.1[j].as_str());
                        j += 1;
                        if j >= var_len {
                            j = 0; //TODO why can this happen? `(" Through ", ["Safety"])`
                        }
                    } else {
                        rebuilt.push(part);
                    }
                }
                GameString::wrap(rebuilt.join(" "))
            };
            state.serialize_field("motto", &rebuilt)?;
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

    fn render(&self, path: &str, _: &GameState, grapher: Option<&Grapher>, _: Option<&GameMap>) {
        if let Some(grapher) = grapher {
            let path = format!("{}/{}/{}.svg", path, Self::get_subdir(), self.id);
            grapher.create_dynasty_graph(self, &path);
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
    fn localize(&mut self, localization: &mut Localizer) {
        if let Some(name) = &self.name {
            self.name = Some(localization.localize(name.as_str()));
        } else {
            return;
        }
        for perk in self.perks.iter_mut() {
            perk.0 = localization.localize(perk.0.as_str());
        }
        if let Some(motto) = &mut self.motto {
            motto.0 = localization.localize(motto.0.as_str());
            for v in motto.1.iter_mut() {
                *v = localization.localize(v.as_str());
            }
        }
    }
}

impl Cullable for Dynasty {
    fn set_depth(&mut self, depth: usize) {
        if depth <= self.depth {
            return;
        }
        self.depth = depth;
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

// TODO add test, especially for dynastic graphs with incest and multiple marriages
// Probably the weirdest comment I've ever written
