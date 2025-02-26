use std::{
    fmt::{self, Display, Formatter},
    path::Path,
    slice::Iter,
    str::FromStr,
};

use jomini::common::{Date, PdsDate};
use serde::Serialize;

use super::{
    super::{
        display::{Grapher, ProceduralPath, Renderable, TreeNode},
        game_data::{GameData, Localizable, LocalizationError, Localize, MapGenerator},
        jinja_env::TITLE_TEMPLATE_NAME,
        parser::{
            GameObjectMap, GameObjectMapping, GameState, ParsingError, SaveFileObject,
            SaveFileValue,
        },
        types::{GameString, Wrapper, WrapperMut},
    },
    Character, Culture, EntityRef, Faith, FromGameObject, GameObjectDerived, GameObjectEntity,
    GameRef,
};

#[derive(Serialize, Debug)]
#[serde(untagged)]
enum TitleType {
    Empire,
    Kingdom,
    Duchy,
    County,
    Barony,
    Other(Option<GameString>),
}

impl<S: AsRef<str>> From<S> for TitleType {
    fn from(value: S) -> Self {
        if let Some(c) = value.as_ref().chars().next() {
            match c {
                'e' => TitleType::Empire,
                'k' => TitleType::Kingdom,
                'd' => TitleType::Duchy,
                'c' => TitleType::County,
                'b' => TitleType::Barony,
                _ => TitleType::Other(None),
            }
        } else {
            TitleType::Other(None)
        }
    }
}

impl Display for TitleType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TitleType::Empire => write!(f, "Empire"),
            TitleType::Kingdom => write!(f, "Kingdom"),
            TitleType::Duchy => write!(f, "Duchy"),
            TitleType::County => write!(f, "County"),
            TitleType::Barony => write!(f, "Barony"),
            TitleType::Other(opt) => write!(f, "{}", opt.as_ref().unwrap_or(&GameString::from(""))),
        }
    }
}

impl Default for TitleType {
    fn default() -> Self {
        TitleType::Other(None)
    }
}

/// A struct representing a title in the game
#[derive(Serialize, Debug)]
pub struct Title {
    key: GameString,
    tier: TitleType,
    name: GameString,
    de_jure: Option<GameRef<Title>>,
    de_facto: Option<GameRef<Title>>,
    de_jure_vassals: Vec<GameRef<Title>>,
    de_facto_vassals: Vec<GameRef<Title>>,
    history: Vec<(Date, Option<GameRef<Character>>, GameString)>,
    claims: Vec<GameRef<Character>>,

    capital: Option<GameRef<Title>>,
    /// Only used for counties
    culture: Option<GameRef<Culture>>,
    /// Only used for counties
    faith: Option<GameRef<Faith>>,
    color: [u8; 3],
}

// TODO enum called Title, that split off county title and higher tiers

impl Title {
    /// Adds a de jure vassal to the title
    pub fn add_jure_vassal(&mut self, vassal: GameRef<Title>) {
        self.de_jure_vassals.push(vassal);
    }

    /// Adds a de facto vassal to the title
    pub fn add_facto_vassal(&mut self, vassal: GameRef<Title>) {
        self.de_facto_vassals.push(vassal);
    }

    /// Recursively gets all the de facto barony keys of the title
    pub fn get_barony_keys(&self) -> Vec<GameString> {
        let mut provinces = Vec::new();
        if let TitleType::Barony = self.tier {
            provinces.push(self.key.clone());
        }
        for v in &self.de_facto_vassals {
            if let Some(v) = v.get_internal().inner() {
                provinces.append(&mut v.get_barony_keys());
            }
        }
        provinces
    }

    pub fn get_de_jure_barony_keys(&self) -> Vec<GameString> {
        let mut provinces = Vec::new();
        if let TitleType::Barony = self.tier {
            provinces.push(self.key.clone());
        }
        for v in &self.de_jure_vassals {
            if let Some(v) = v.get_internal().inner() {
                provinces.append(&mut v.get_de_jure_barony_keys());
            }
        }
        provinces
    }

    /// Returns the key of the title
    pub fn get_key(&self) -> GameString {
        self.key.clone()
    }

    /// Returns an iterator over the history of the title
    pub fn get_history_iter(&self) -> Iter<(Date, Option<GameRef<Character>>, GameString)> {
        self.history.iter()
    }

    /// Returns the capital of the title
    pub fn get_capital(&self) -> Option<GameRef<Title>> {
        self.capital.clone()
    }

    /// Adds the culture and faith data to the title
    pub fn add_county_data(&mut self, culture: GameRef<Culture>, faith: GameRef<Faith>) {
        if let TitleType::County = self.tier {
            self.culture = Some(culture);
            self.faith = Some(faith);
        } else {
            panic!("Can only add county data to a county title");
        }
    }

    /// Returns the culture of the title
    pub fn get_culture(&self) -> Option<GameRef<Culture>> {
        if let Some(culture) = &self.culture {
            return Some(culture.clone());
        } else {
            return None;
        }
    }

    /// Returns the faith of the title
    pub fn get_faith(&self) -> Option<GameRef<Faith>> {
        if let Some(faith) = &self.faith {
            return Some(faith.clone());
        } else {
            return None;
        }
    }

    /// Returns the holder of the title
    pub fn get_holder(&self) -> Option<GameRef<Character>> {
        if let Some(entry) = self.history.last() {
            return entry.1.clone();
        }
        None
    }
}

impl FromGameObject for Title {
    fn from_game_object(
        base: &GameObjectMap,
        game_state: &mut GameState,
    ) -> Result<Self, ParsingError> {
        let key = base.get_string("key")?;
        let mut title = Self {
            key: key.clone(),
            tier: TitleType::from(key),
            name: base.get_string("name")?,
            color: base
                .get("color")
                .map(|v| v.as_object().and_then(|obj| obj.as_array()))
                .transpose()?
                .map_or([70, 255, 70], |color_obj| {
                    [
                        color_obj[0].as_integer().unwrap() as u8,
                        color_obj[1].as_integer().unwrap() as u8,
                        color_obj[2].as_integer().unwrap() as u8,
                    ]
                }),
            de_jure: base
                .get("de_jure_liege")
                .map(|liege| {
                    liege
                        .as_id()
                        .and_then(|liege_id| Ok(game_state.get_title(&liege_id)))
                })
                .transpose()?,
            de_facto: base
                .get("de_facto_liege")
                .map(|liege| {
                    liege
                        .as_id()
                        .and_then(|liege_id| Ok(game_state.get_title(&liege_id).clone()))
                })
                .transpose()?,
            de_jure_vassals: Vec::default(),
            de_facto_vassals: Vec::default(),
            history: Vec::default(),
            claims: Vec::default(),
            capital: base
                .get("capital")
                .map(|capital| capital.as_id().and_then(|id| Ok(game_state.get_title(&id))))
                .transpose()?,
            culture: None,
            faith: None,
        };
        if let Some(claims) = base.get("claim") {
            if let SaveFileValue::Object(claims) = claims {
                for claim in claims.as_array()? {
                    title
                        .claims
                        .push(game_state.get_character(&claim.as_id()?).clone());
                }
            } else {
                title
                    .claims
                    .push(game_state.get_character(&claims.as_id()?).clone());
            }
        }

        if let Some(hist) = base.get("history") {
            for (h, val) in hist.as_object()?.as_map()? {
                let character;
                let action: GameString;
                if let SaveFileValue::Object(o) = val {
                    match o {
                        SaveFileObject::Array(arr) => {
                            for entry in arr {
                                let loc_action;
                                let loc_character;
                                if let SaveFileValue::Object(o) = entry {
                                    let o = o.as_map()?;
                                    loc_action = o.get_string("type")?;
                                    if let Some(holder) = o.get("holder") {
                                        loc_character = Some(
                                            game_state.get_character(&holder.as_id()?).clone(),
                                        );
                                    } else {
                                        loc_character = None;
                                    }
                                } else {
                                    loc_action = GameString::from("Inherited");
                                    loc_character =
                                        Some(game_state.get_character(&entry.as_id()?).clone());
                                }
                                title
                                    .history
                                    .push((Date::from_str(h)?, loc_character, loc_action))
                            }
                            continue; //if it's an array we handled all the adding already in the loop above
                        }
                        SaveFileObject::Map(o) => {
                            action = o.get_string("type")?;
                            match o.get("holder") {
                                Some(h) => {
                                    character = Some(game_state.get_character(&h.as_id()?).clone());
                                }
                                None => {
                                    character = None;
                                }
                            }
                        }
                    }
                } else {
                    action = GameString::from("Inherited");
                    character = Some(game_state.get_character(&val.as_id()?).clone());
                }
                title.history.push((Date::from_str(h)?, character, action));
            }
        }
        //sort history by the first element of the tuple (the date) in descending order
        title.history.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(title)
    }
}

impl GameObjectDerived for Title {
    fn get_name(&self) -> GameString {
        self.name.clone()
    }

    fn get_references<E: From<EntityRef>, C: Extend<E>>(&self, collection: &mut C) {
        if let Some(de_jure) = &self.de_jure {
            collection.extend([E::from(de_jure.clone().into())]);
        }
        if let Some(de_facto) = &self.de_facto {
            collection.extend([E::from(de_facto.clone().into())]);
        }
        for v in &self.de_jure_vassals {
            collection.extend([E::from(v.clone().into())]);
        }
        for v in &self.de_facto_vassals {
            collection.extend([E::from(v.clone().into())]);
        }
        for c in &self.claims {
            collection.extend([E::from(c.clone().into())]);
        }
        if let Some(capital) = &self.capital {
            collection.extend([E::from(capital.clone().into())]);
        }
    }

    fn finalize(&mut self, reference: &GameRef<Title>) {
        if let Some(de_jure) = &self.de_jure {
            if let Some(de_jure) = de_jure.get_internal_mut().inner_mut() {
                de_jure.add_jure_vassal(reference.clone());
            }
        }
        if let Some(de_facto) = &self.de_facto {
            if let Some(de_facto) = de_facto.get_internal_mut().inner_mut() {
                de_facto.add_facto_vassal(reference.clone());
            }
        }
    }
}

impl TreeNode<Vec<GameRef<Title>>> for Title {
    fn get_children(&self) -> Option<Vec<GameRef<Title>>> {
        if self.de_jure_vassals.is_empty() {
            return None;
        }
        Some(self.de_jure_vassals.clone())
    }

    fn get_class(&self) -> Option<GameString> {
        if let TitleType::Other(opt) = &self.tier {
            return opt.clone();
        } else {
            return Some(GameString::from(self.tier.to_string()));
        }
    }

    fn get_parent(&self) -> Option<Vec<GameRef<Title>>> {
        if let Some(de_jure) = &self.de_jure {
            return Some(vec![de_jure.clone()]);
        }
        None
    }
}

impl ProceduralPath for Title {
    fn get_subdir() -> &'static str {
        "titles"
    }
}

impl Renderable for GameObjectEntity<Title> {
    fn get_template() -> &'static str {
        TITLE_TEMPLATE_NAME
    }

    fn render(&self, path: &Path, game_state: &GameState, _: Option<&Grapher>, data: &GameData) {
        if let Some(map) = data.get_map() {
            if let Some(title) = self.inner() {
                if title.de_facto_vassals.len() == 0 {
                    return;
                }
                let mut buf = path.join(Title::get_subdir());
                buf.push(format!("{}.png", self.id));
                let mut title_map = map.create_map_flat(title.get_barony_keys(), title.color);
                title_map.draw_text(format!(
                    "{} at {}",
                    title.name,
                    game_state.get_current_date().unwrap().iso_8601()
                ));
                title_map.save(&buf);
            }
        }
    }
}

impl Localizable for Title {
    fn localize<L: Localize<GameString>>(
        &mut self,
        localization: &mut L,
    ) -> Result<(), LocalizationError> {
        if self.name == self.key {
            self.name = localization.localize(&self.key)?;
        }
        //for o in self.history.iter_mut() {
        //    o.2 = localization.localize(o.2.as_str());
        //}
        Ok(())
    }
}
