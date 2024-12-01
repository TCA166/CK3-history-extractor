use serde::{ser::SerializeStruct, Serialize};

use super::{
    super::{
        display::{
            Cullable, GameMap, Grapher, Localizable, Localizer, Renderable, RenderableType,
            TreeNode,
        },
        jinja_env::C_TEMPLATE_NAME,
        parser::{GameObjectMap, GameState, GameString, SaveFileValue},
        types::{OneOrMany, Wrapper, WrapperMut},
    },
    serialize_array, Artifact, Culture, DerivedRef, DummyInit, Dynasty, Faith, GameId,
    GameObjectDerived, Memory, Shared, Title,
};

/// An enum that holds either a character or a reference to a character.
/// Effectively either a vassal([Character]) or a vassal([DerivedRef]) contract.
/// This is done so that we can hold a reference to a vassal contract, and also manually added characters from vassals registering themselves via [Character::add_vassal].
enum Vassal {
    Character(Shared<Character>),
    Reference(Shared<DerivedRef<Character>>),
}

impl GameObjectDerived for Vassal {
    fn get_id(&self) -> GameId {
        match self {
            Vassal::Character(c) => c.get_internal().get_id(),
            Vassal::Reference(r) => r.get_internal().get_ref().get_internal().get_id(),
        }
    }

    fn get_name(&self) -> GameString {
        match self {
            Vassal::Character(c) => c.get_internal().get_name(),
            Vassal::Reference(r) => r.get_internal().get_ref().get_internal().get_name(),
        }
    }
}

impl Serialize for Vassal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Vassal::Character(c) => c.serialize(serializer),
            Vassal::Reference(r) => r.serialize(serializer),
        }
    }
}

impl Cullable for Vassal {
    fn is_ok(&self) -> bool {
        match self {
            Vassal::Character(c) => c.get_internal().is_ok(),
            Vassal::Reference(r) => r.get_internal().get_ref().get_internal().is_ok(),
        }
    }

    fn get_depth(&self) -> usize {
        match self {
            Vassal::Character(c) => c.get_internal().get_depth(),
            Vassal::Reference(r) => r.get_internal().get_ref().get_internal().get_depth(),
        }
    }

    fn set_depth(&mut self, depth: usize) {
        match self {
            Vassal::Character(c) => {
                if let Ok(mut o) = c.try_get_internal_mut() {
                    o.set_depth(depth);
                }
            }
            Vassal::Reference(r) => {
                if let Ok(mut o) = r.get_internal().get_ref().try_get_internal_mut() {
                    o.set_depth(depth);
                }
            }
        }
    }
}

impl Renderable for Vassal {
    fn get_template() -> &'static str {
        Character::get_template()
    }

    fn get_subdir() -> &'static str {
        Character::get_subdir()
    }

    fn render(
        &self,
        path: &str,
        game_state: &GameState,
        grapher: Option<&Grapher>,
        map: Option<&GameMap>,
    ) {
        match self {
            Vassal::Character(c) => c.get_internal().render(path, game_state, grapher, map),
            Vassal::Reference(r) => r
                .get_internal()
                .get_ref()
                .get_internal()
                .render(path, game_state, grapher, map),
        }
    }

    fn append_ref(&self, stack: &mut Vec<RenderableType>) {
        match self {
            Vassal::Character(c) => stack.push(RenderableType::Character(c.clone())),
            Vassal::Reference(r) => stack.push(RenderableType::Character(
                r.get_internal().get_ref().clone(),
            )),
        }
    }
}

/// Represents a character in the game.
/// Implements [GameObjectDerived], [Renderable] and [Cullable].
pub struct Character {
    id: GameId,
    name: Option<GameString>,
    nick: Option<GameString>,
    birth: Option<GameString>,
    dead: bool,
    date: Option<GameString>,
    reason: Option<GameString>,
    faith: Option<Shared<Faith>>,
    culture: Option<Shared<Culture>>,
    house: Option<Shared<Dynasty>>,
    skills: Vec<i8>,
    traits: Vec<GameString>,
    recessive: Vec<GameString>,
    spouses: Vec<Shared<Character>>,
    former: Vec<Shared<Character>>,
    children: Vec<Shared<Character>>,
    parents: Vec<Shared<Character>>,
    dna: Option<GameString>,
    memories: Vec<Shared<Memory>>,
    titles: Vec<Shared<Title>>,
    gold: f32,
    piety: f32,
    prestige: f32,
    dread: f32,
    strength: f32,
    kills: Vec<Shared<Character>>,
    languages: Vec<GameString>,
    vassals: Vec<Vassal>,
    liege: Option<DerivedRef<Character>>,
    female: bool,
    depth: usize,
    artifacts: Vec<Shared<Artifact>>,
}

// So both faith and culture can be stored for a character in the latest leader of their house.
// The problem with reading that now is that while Houses are already likely loaded,
// the characters that the houses hold reference to are likely still dummy, so we can't read the faith and culture from the house leader.
// So we will be returning None for now in case either is missing, but later during serialization read the one from house.

/// Gets the faith of the character
fn get_faith(base: &GameObjectMap, game_state: &mut GameState) -> Option<Shared<Faith>> {
    if let Some(faith_node) = base.get("faith") {
        return Some(game_state.get_faith(&faith_node.as_id()).clone());
    }
    None
}

/// Gets the culture of the character
fn get_culture(base: &GameObjectMap, game_state: &mut GameState) -> Option<Shared<Culture>> {
    if let Some(culture_node) = base.get("culture") {
        return Some(game_state.get_culture(&culture_node.as_id()).clone());
    }
    None
}

/// Gets the skills of the character
fn get_skills(skills: &mut Vec<i8>, base: &GameObjectMap) {
    for s in base.get_object_ref("skill").as_array().into_iter() {
        skills.push(s.as_string().parse::<i8>().unwrap());
    }
}

/// Parses the dead_data field of the character
fn get_dead(
    dead: &mut bool,
    reason: &mut Option<GameString>,
    data: &mut Option<GameString>,
    titles: &mut Vec<Shared<Title>>,
    liege: &mut Option<DerivedRef<Character>>,
    self_id: &GameId,
    base: &GameObjectMap,
    game_state: &mut GameState,
) {
    if let Some(dead_data) = base.get("dead_data") {
        *dead = true;
        let o = dead_data.as_object().as_map();
        if let Some(reason_node) = o.get("reason") {
            *reason = Some(reason_node.as_string());
        }
        if let Some(domain_node) = o.get("domain") {
            for t in domain_node.as_object().as_array().into_iter() {
                titles.push(game_state.get_title(&t.as_id()));
            }
        }
        *data = Some(o.get("date").unwrap().as_string());
        if let Some(liege_node) = o.get("liege") {
            let id = liege_node.as_id();
            if id == *self_id {
                return;
            }
            let liege_chr = game_state.get_character(&id).clone();
            if let Ok(mut o) = liege_chr.try_get_internal_mut() {
                o.add_vassal(game_state.get_character(self_id));
            }
            *liege = Some(DerivedRef::<Character>::from_derived(liege_chr.clone()));
        }
    }
}

/// Gets the recessive traits of the character
fn get_recessive(recessive: &mut Vec<GameString>, base: &GameObjectMap) {
    if let Some(rec_t) = base.get("recessive_traits") {
        for r in rec_t.as_object().as_array() {
            recessive.push(r.as_string());
        }
    }
}

/// Parses the family_data field of the character
fn get_family(
    self_id: GameId,
    spouses: &mut Vec<Shared<Character>>,
    former_spouses: &mut Vec<Shared<Character>>,
    children: &mut Vec<Shared<Character>>,
    base: &GameObjectMap,
    game_state: &mut GameState,
) {
    if let Some(family_data) = base.get("family_data") {
        let f = family_data.as_object();
        if f.is_empty() {
            return;
        }
        let f = f.as_map();
        if let Some(former_spouses_node) = f.get("former_spouses") {
            for s in former_spouses_node.as_object().as_array() {
                former_spouses.push(game_state.get_character(&s.as_id()).clone());
            }
        }
        if let Some(spouse_node) = f.get("spouse") {
            match spouse_node {
                SaveFileValue::Object(o) => {
                    for s in o.as_array() {
                        let c = game_state.get_character(&s.as_id()).clone();
                        let contains = former_spouses
                            .iter()
                            .any(|x| x.get_internal().get_id() == c.get_internal().get_id());
                        if !contains {
                            spouses.push(c);
                        }
                    }
                }
                SaveFileValue::String(o) => {
                    let c = game_state
                        .get_character(&o.parse::<GameId>().unwrap())
                        .clone();
                    let contains = former_spouses
                        .iter()
                        .any(|x| x.get_internal().get_id() == c.get_internal().get_id());
                    if !contains {
                        spouses.push(c);
                    }
                }
            }
        }
        if let Some(primary_spouse_node) = f.get("primary_spouse") {
            let c = game_state
                .get_character(&primary_spouse_node.as_id())
                .clone();
            let mut contains = former_spouses
                .iter()
                .any(|x| x.get_internal().get_id() == c.get_internal().get_id());
            contains = contains
                || spouses
                    .iter()
                    .any(|x| x.get_internal().get_id() == c.get_internal().get_id());
            if !contains {
                spouses.push(c);
            }
        }
        if let Some(children_node) = f.get("child") {
            let parent = game_state.get_character(&self_id);
            for s in children_node.as_object().as_array() {
                let c = game_state.get_character(&s.as_id()).clone();
                c.get_internal_mut().register_parent(parent.clone());
                children.push(c);
            }
        }
    }
}

/// Gets the traits of the character
fn get_traits(traits: &mut Vec<GameString>, base: &GameObjectMap, game_state: &mut GameState) {
    if let Some(traits_node) = base.get("traits") {
        for t in traits_node.as_object().as_array() {
            let index = t.as_string().parse::<u16>().unwrap();
            traits.push(game_state.get_trait(index));
        }
    }
}

/// Processes a currency node of the character
fn process_currency(currency_node: Option<&SaveFileValue>) -> f32 {
    match currency_node {
        Some(o) => {
            if let Some(currency) = o.as_object().as_map().get("accumulated") {
                currency.as_string().parse::<f32>().unwrap()
            } else {
                0.0
            }
        }
        None => 0.0,
    }
}

/// Parses the alive_data field of the character
fn parse_alive_data(
    base: &GameObjectMap,
    piety: &mut f32,
    prestige: &mut f32,
    gold: &mut f32,
    kills: &mut Vec<Shared<Character>>,
    languages: &mut Vec<GameString>,
    traits: &mut Vec<GameString>,
    memories: &mut Vec<Shared<Memory>>,
    artifacts: &mut Vec<Shared<Artifact>>,
    game_state: &mut GameState,
) {
    if let Some(alive_data) = base.get("alive_data") {
        let alive_data = alive_data.as_object().as_map();
        *piety = process_currency(alive_data.get("piety"));
        *prestige = process_currency(alive_data.get("prestige"));
        if let Some(kills_node) = alive_data.get("kills") {
            for k in kills_node.as_object().as_array() {
                kills.push(game_state.get_character(&k.as_id()).clone());
            }
        }
        if let Some(gold_node) = alive_data.get("gold") {
            *gold = gold_node.as_string().parse::<f32>().unwrap();
        }
        for l in alive_data.get_object_ref("languages").as_array() {
            languages.push(l.as_string());
        }
        if let Some(perk_node) = alive_data.get("perks") {
            for p in perk_node.as_object().as_array() {
                traits.push(p.as_string());
            }
        }
        if let Some(memory_node) = alive_data.get("memories") {
            for m in memory_node.as_object().as_array() {
                memories.push(game_state.get_memory(&m.as_id()).clone());
            }
        }
        if let Some(inventory_node) = alive_data.get("inventory") {
            if let Some(artifacts_node) = inventory_node.as_object().as_map().get("artifacts") {
                for a in artifacts_node.as_object().as_array() {
                    artifacts.push(game_state.get_artifact(&a.as_id()).clone());
                }
            }
        }
    }
}

/// Parses the landed_data field of the character
fn get_landed_data(
    dread: &mut f32,
    strength: &mut f32,
    titles: &mut Vec<Shared<Title>>,
    vassals: &mut Vec<Vassal>,
    base: &GameObjectMap,
    game_state: &mut GameState,
) {
    if let Some(landed_data_node) = base.get("landed_data") {
        let landed_data = landed_data_node.as_object().as_map();
        if let Some(dread_node) = landed_data.get("dread") {
            *dread = dread_node.as_string().parse::<f32>().unwrap();
        }
        if let Some(strength_node) = landed_data.get("strength") {
            *strength = strength_node.as_string().parse::<f32>().unwrap();
        }
        if let Some(titles_node) = landed_data.get("domain") {
            for t in titles_node.as_object().as_array() {
                titles.push(game_state.get_title(&t.as_id()));
            }
        }
        if let Some(vassals_node) = landed_data.get("vassal_contracts") {
            for v in vassals_node.as_object().as_array() {
                vassals.push(Vassal::Reference(game_state.get_vassal(&v.as_id())));
            }
        }
    }
}

/// Gets the dynasty of the character
fn get_dynasty(
    base: &GameObjectMap,
    game_state: &mut GameState,
    self_id: &GameId,
) -> Option<Shared<Dynasty>> {
    if let Some(dynasty_id) = base.get("dynasty_house") {
        let d = game_state.get_dynasty(&dynasty_id.as_id());
        d.get_internal_mut()
            .register_member(game_state.get_character(&self_id));
        return Some(d);
    }
    None
}

/// Gets the nickname of the character
fn get_nick(base: &GameObjectMap) -> Option<GameString> {
    // this feature is new i think with roads to power, but very handy
    if let Some(text) = base.get("nickname_text") {
        return Some(text.as_string());
    }
    if let Some(nick) = base.get("nickname") {
        return Some(nick.as_string());
    }
    None
}

impl Character {
    /// Gets the faith of the character
    pub fn get_faith(&self) -> Option<Shared<Faith>> {
        if let Some(faith) = &self.faith {
            return Some(faith.clone());
        } else {
            if let Some(house) = &self.house {
                let o = house.get_internal();
                if let Some(faith) = o.get_faith() {
                    return Some(faith.clone());
                }
            }
        }
        None
    }

    /// Gets the culture of the character
    pub fn get_culture(&self) -> Option<Shared<Culture>> {
        if let Some(culture) = &self.culture {
            return Some(culture.clone());
        } else {
            if let Some(house) = &self.house {
                let o = house.get_internal();
                if let Some(culture) = o.get_culture() {
                    return Some(culture.clone());
                }
            }
        }
        None
    }

    /// Adds a character as a parent of this character
    pub fn register_parent(&mut self, parent: Shared<Character>) {
        self.parents.push(parent);
    }

    /// Gets the death date string of the character
    pub fn get_death_date(&self) -> Option<GameString> {
        self.date.clone()
    }

    /// Adds a character as a vassal of this character
    pub fn add_vassal(&mut self, vassal: Shared<Character>) {
        self.vassals.push(Vassal::Character(vassal));
    }

    /* TODO implement this.
    The problem is that we cannot do this in the init, because the vassals aren't loaded, and within cull we don't have a shared reference

    pub fn set_liege(&mut self, liege:Shared<Character>){
        self.liege = Some(DerivedRef::from_derived(liege));
    }
    */

    /// Gets all of the held de jure barony keys of the character and their vassals
    pub fn get_barony_keys(&self, de_jure: bool) -> Vec<GameString> {
        let mut provinces = Vec::new();
        for title in self.titles.iter() {
            let title = title.get_internal();
            let key = title.get_key().unwrap();
            if key.starts_with("e_") || key.starts_with("k_") {
                //for kingdoms and empires we don't want to add the de jure baronies
                continue;
            } else {
                if de_jure {
                    provinces.append(&mut title.get_de_jure_barony_keys());
                } else {
                    provinces.append(&mut title.get_barony_keys());
                }
            }
        }
        for vassal in self.vassals.iter() {
            match vassal {
                Vassal::Character(c) => {
                    provinces.append(&mut c.get_internal().get_barony_keys(de_jure))
                }
                Vassal::Reference(c) => provinces.append(
                    &mut c
                        .get_internal()
                        .get_ref()
                        .get_internal()
                        .get_barony_keys(de_jure),
                ),
            }
        }
        return provinces;
    }

    /// Gets the DNA similarity of the character with another character
    pub fn dna_similarity(&self, other: Shared<Character>) -> f32 {
        if let Some(dna) = &self.dna {
            // Ok so I did discover that dna string lengths are always multiple of 4, which does imply that they are encoded ints or shorts
            // but that seems to be just a coincidence? experimenting with comparing multiples of 4 characters doesn't seem to give any meaningful results
            // so single char comparison it is, the metric is still far from perfect, but until we know more about the encoding, it's the best we can do
            let mut dna_chars = dna.chars();
            let other = other.get_internal();
            let mut other_chars = other.dna.as_ref().unwrap().chars();
            let mut similarity = 0;
            while let (Some(d), Some(o)) = (dna_chars.next(), other_chars.next()) {
                if d == o {
                    similarity += 1;
                }
            }
            return similarity as f32 / dna.len() as f32;
        } else {
            return 0.0;
        }
    }

    /// Gets the descendants of the character
    pub fn get_descendants(&self) -> Vec<Shared<Character>> {
        let mut res = Vec::new();
        let mut stack: Vec<Shared<Character>> = Vec::new();
        for child in self.children.iter() {
            stack.push(child.clone());
            res.push(child.clone());
        }
        while !stack.is_empty() {
            let c = stack.pop().unwrap();
            let c = c.get_internal();
            for child in c.children.iter() {
                stack.push(child.clone());
                res.push(child.clone());
            }
        }
        return res;
    }

    /// Gets the dynasty of the character
    pub fn get_dynasty(&self) -> Option<Shared<Dynasty>> {
        if let Some(house) = &self.house {
            return Some(house.clone());
        } else {
            return None;
        }
    }
}

impl DummyInit for Character {
    fn dummy(id: GameId) -> Self {
        Character {
            name: None,
            nick: None,
            birth: None,
            dead: false,
            date: None,
            reason: None,
            faith: None,
            culture: None,
            house: None,
            skills: Vec::new(),
            traits: Vec::new(),
            recessive: Vec::new(),
            spouses: Vec::new(),
            former: Vec::new(),
            children: Vec::new(),
            parents: Vec::new(),
            dna: None,
            memories: Vec::new(),
            titles: Vec::new(),
            gold: 0.0,
            piety: 0.0,
            prestige: 0.0,
            dread: 0.0,
            strength: 0.0,
            kills: Vec::new(),
            languages: Vec::new(),
            vassals: Vec::new(),
            liege: None,
            female: false,
            id: id,
            depth: 0,
            artifacts: Vec::new(),
        }
    }

    fn init(&mut self, base: &GameObjectMap, game_state: &mut GameState) {
        if let Some(female) = base.get("female") {
            self.female = female.as_string().as_str() == "yes";
        }
        get_dead(
            &mut self.dead,
            &mut self.reason,
            &mut self.date,
            &mut self.titles,
            &mut self.liege,
            &self.id,
            &base,
            game_state,
        );
        //find skills
        get_skills(&mut self.skills, &base);
        //find recessive traits
        get_recessive(&mut self.recessive, &base);
        get_family(
            self.id,
            &mut self.spouses,
            &mut self.former,
            &mut self.children,
            &base,
            game_state,
        );
        //find dna
        let dna = match base.get("dna") {
            Some(d) => Some(d.as_string()),
            None => None,
        };
        //find traits
        get_traits(&mut self.traits, &base, game_state);
        //find alive data
        if !self.dead {
            parse_alive_data(
                &base,
                &mut self.piety,
                &mut self.prestige,
                &mut self.gold,
                &mut self.kills,
                &mut self.languages,
                &mut self.traits,
                &mut self.memories,
                &mut self.artifacts,
                game_state,
            );
        }
        //find landed data
        get_landed_data(
            &mut self.dread,
            &mut self.strength,
            &mut self.titles,
            &mut self.vassals,
            &base,
            game_state,
        );
        //find house
        self.name = Some(base.get("first_name").unwrap().as_string());
        self.nick = get_nick(&base);
        self.birth = Some(base.get("birth").unwrap().as_string());
        self.house = get_dynasty(&base, game_state, &self.id);
        self.faith = get_faith(&base, game_state);
        self.culture = get_culture(&base, game_state);
        self.dna = dna;
    }
}

impl GameObjectDerived for Character {
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        if self.name.is_none() {
            return GameString::wrap("Unknown".to_string());
        }
        self.name.as_ref().unwrap().clone()
    }
}

impl TreeNode for Character {
    fn get_children(&self) -> Option<OneOrMany<Character>> {
        if self.children.is_empty() {
            return None;
        }
        Some(OneOrMany::Many(&self.children))
    }

    fn get_parent(&self) -> Option<OneOrMany<Character>> {
        if self.parents.is_empty() {
            return None;
        }
        Some(OneOrMany::Many(&self.parents))
    }

    fn get_class(&self) -> Option<GameString> {
        if let Some(house) = &self.house {
            return Some(house.get_internal().get_name().clone());
        } else {
            None
        }
    }
}

impl Serialize for Character {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Character", 30)?;
        state.serialize_field("name", &self.get_name())?;
        state.serialize_field("nick", &self.nick)?;
        state.serialize_field("birth", &self.birth)?;
        state.serialize_field("dead", &self.dead)?;
        state.serialize_field("date", &self.date)?;
        state.serialize_field("reason", &self.reason)?;
        let mut faith = self.faith.clone();
        let mut culture = self.culture.clone();
        if let Some(house) = &self.house {
            let rd = DerivedRef::<Dynasty>::from_derived(house.clone());
            state.serialize_field("house", &rd)?;
            if faith.is_none() {
                let o = house.get_internal();
                faith = o.get_faith();
            }
            if culture.is_none() {
                let o = house.get_internal();
                culture = o.get_culture();
            }
        }
        if let Some(faith) = faith {
            let rf = DerivedRef::<Faith>::from_derived(faith.clone());
            state.serialize_field("faith", &rf)?;
        }
        if let Some(culture) = culture {
            let rc = DerivedRef::<Culture>::from_derived(culture.clone());
            state.serialize_field("culture", &rc)?;
        }
        state.serialize_field("skills", &self.skills)?;
        state.serialize_field("traits", &self.traits)?;
        state.serialize_field("recessive", &self.recessive)?;
        //serialize spouses as DerivedRef
        let spouses = serialize_array::<Character>(&self.spouses);
        state.serialize_field("spouses", &spouses)?;
        //serialize former as DerivedRef
        let former = serialize_array::<Character>(&self.former);
        state.serialize_field("former", &former)?;
        //serialize children as DerivedRef
        let children = serialize_array::<Character>(&self.children);
        state.serialize_field("children", &children)?;
        //serialize parents as DerivedRef
        let parents = serialize_array::<Character>(&self.parents);
        state.serialize_field("parents", &parents)?;
        state.serialize_field("dna", &self.dna)?;
        //serialize memories as DerivedRef
        state.serialize_field("memories", &self.memories)?;
        //serialize titles as DerivedRef
        let titles = serialize_array::<Title>(&self.titles);
        state.serialize_field("titles", &titles)?;
        state.serialize_field("gold", &self.gold)?;
        state.serialize_field("piety", &self.piety)?;
        state.serialize_field("prestige", &self.prestige)?;
        state.serialize_field("dread", &self.dread)?;
        state.serialize_field("strength", &self.strength)?;
        //serialize kills as DerivedRef
        let kills = serialize_array::<Character>(&self.kills);
        state.serialize_field("kills", &kills)?;
        state.serialize_field("languages", &self.languages)?;
        let mut vassals = Vec::new();
        for vassal in self.vassals.iter() {
            match vassal {
                Vassal::Character(c) => {
                    vassals.push(DerivedRef::from_derived(c.clone()));
                }
                Vassal::Reference(c) => {
                    vassals.push(DerivedRef::from_derived(c.get_internal().get_ref().clone()))
                }
            }
        }
        state.serialize_field("artifacts", &self.artifacts)?;
        state.serialize_field("vassals", &vassals)?;
        state.serialize_field("id", &self.id)?;
        if let Some(liege) = &self.liege {
            state.serialize_field("liege", liege)?;
        }
        state.end()
    }
}

impl Renderable for Character {
    fn get_template() -> &'static str {
        C_TEMPLATE_NAME
    }

    fn get_subdir() -> &'static str {
        "characters"
    }

    fn append_ref(&self, stack: &mut Vec<RenderableType>) {
        if let Some(faith) = &self.faith {
            stack.push(RenderableType::Faith(faith.clone()));
        }
        if let Some(culture) = &self.culture {
            stack.push(RenderableType::Culture(culture.clone()));
        }
        if let Some(house) = &self.house {
            stack.push(RenderableType::Dynasty(house.clone()));
        }
        if let Some(liege) = &self.liege {
            stack.push(RenderableType::Character(liege.get_ref().clone()));
        }
        for s in self.spouses.iter() {
            stack.push(RenderableType::Character(s.clone()));
        }
        for s in self.former.iter() {
            stack.push(RenderableType::Character(s.clone()));
        }
        for s in self.children.iter() {
            stack.push(RenderableType::Character(s.clone()));
        }
        for s in self.parents.iter() {
            stack.push(RenderableType::Character(s.clone()));
        }
        for s in self.kills.iter() {
            stack.push(RenderableType::Character(s.clone()));
        }
        for s in self.vassals.iter() {
            match s {
                Vassal::Character(c) => stack.push(RenderableType::Character(c.clone())),
                Vassal::Reference(c) => stack.push(RenderableType::Character(
                    c.get_internal().get_ref().clone(),
                )),
            }
        }
        for s in self.titles.iter() {
            stack.push(RenderableType::Title(s.clone()));
        }
        for m in self.memories.iter() {
            m.get_internal().add_participants(stack);
        }
        for a in self.artifacts.iter() {
            a.get_internal().add_ref(stack);
        }
    }
}

impl Localizable for Character {
    fn localize(&mut self, localization: &mut Localizer) {
        if self.name.is_none() {
            return;
        } else {
            self.name = Some(localization.localize(self.name.as_ref().unwrap().as_str()));
        }
        if let Some(nick) = &self.nick {
            self.nick = Some(localization.localize(nick.as_str()));
        }
        if let Some(reason) = &self.reason {
            self.reason = Some(localization.localize(reason.as_str()));
        }
        for t in self.traits.iter_mut() {
            *t = localization.localize(t.as_str());
        }
        for t in self.languages.iter_mut() {
            *t = localization.localize(t.as_str());
        }
    }
}

impl Cullable for Character {
    fn set_depth(&mut self, depth: usize) {
        if depth <= self.depth {
            return;
        }
        //cullable set
        self.depth = depth;
        if let Some(liege) = &self.liege {
            if let Ok(mut o) = liege.get_ref().try_get_internal_mut() {
                o.set_depth(depth - 1);
            }
        }
        for s in self.spouses.iter() {
            if let Ok(mut o) = s.try_get_internal_mut() {
                o.set_depth(depth - 1);
            }
        }
        for s in self.former.iter() {
            if let Ok(mut o) = s.try_get_internal_mut() {
                o.set_depth(depth - 1);
            }
        }
        for s in self.children.iter() {
            if let Ok(mut o) = s.try_get_internal_mut() {
                o.set_depth(depth - 1);
            }
        }
        for s in self.parents.iter() {
            if let Ok(mut o) = s.try_get_internal_mut() {
                o.set_depth(depth - 1);
            }
        }
        for s in self.kills.iter() {
            if let Ok(mut o) = s.try_get_internal_mut() {
                o.set_depth(depth - 1);
            }
        }
        for s in self.vassals.iter_mut() {
            s.set_depth(depth);
        }
        if let Some(culture) = &self.culture {
            if let Ok(mut o) = culture.try_get_internal_mut() {
                o.set_depth(depth);
            }
        }
        if let Some(faith) = &self.faith {
            if let Ok(mut o) = faith.try_get_internal_mut() {
                o.set_depth(depth - 1);
            }
        }
        for s in self.titles.iter() {
            if let Ok(mut o) = s.try_get_internal_mut() {
                o.set_depth(depth - 1);
            }
        }
        for s in self.memories.iter() {
            s.get_internal_mut().set_depth(depth);
        }
        //sort so that most worthy artifacts are shown first
        self.artifacts.sort();
        self.artifacts.reverse();
        for s in self.artifacts.iter() {
            if let Ok(mut o) = s.try_get_internal_mut() {
                o.set_depth(depth - 1);
            }
        }
        if let Some(house) = &self.house {
            if let Ok(mut o) = house.try_get_internal_mut() {
                o.set_depth(depth - 1);
            }
        }
    }

    fn get_depth(&self) -> usize {
        self.depth
    }
}
