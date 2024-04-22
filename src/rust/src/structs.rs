mod renderer;
use renderer::renderable;

struct Faith {

}

struct Culture {

}

struct Dynasty {

}

struct Memory {

}

struct Title {

}

struct Character {
    name: &String,
    nick: &String,
    birth: &String,
    dead: &String,
    date: Option<&String>,
    reason: Option<&String>,
    faith: &Faith,
    culture: &Culture,
    house: &Dynasty,
    skills: Vec<u8>,
    traits: Vec<&String>,
    recessive: Vec<&String>,
    spouses: Vec<&Character>,
    former: Vec<&Character>,
    children: Vec<&Character>,
    dna: &String,
    memories: Vec<&Memory>,
    titles: Vec<&Title>,
    gold: u32,
    piety: u32,
    prestige: u32,
    dread: u32,
    strength: u32,
    kills: Vec<&Character>,
    languages: Vec<&String>,
    vassals: Vec<&Character>
}
