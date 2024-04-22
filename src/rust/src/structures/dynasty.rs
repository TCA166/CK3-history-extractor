use minijinja::{Environment, context};

use serde::Serialize;
use serde::ser::SerializeStruct;

use super::renderer::Renderable;

pub struct Dynasty<'a>{
    parent: &'a Dynasty<'a>,
    name: &'a String,
    
}