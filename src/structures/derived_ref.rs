use serde::Serialize;
use serde::ser::SerializeStruct;

use super::super::{display::{Localizer, Cullable, Renderable, Renderer, Grapher, GameMap}, types::WrapperMut};
use super::{GameId, GameObjectDerived, Shared, Wrapper};

/// A shallow serializable reference to a derived game object.
/// The idea is to provide the id and name of the object, without serializing the whole object.
/// This is useful for serializing references to objects that are not in the current scope.
pub struct DerivedRef<T> where T:Renderable{
    id: GameId,
    obj: Option<Shared<T>>
}

impl<T> DerivedRef<T> where T:Renderable{
    /// Create a new DerivedRef from a [Shared] object.
    /// This will clone the object and store a reference to it.
    pub fn from_derived(obj:Shared<T>) -> Self{
        let o = obj.get_internal();
        DerivedRef{
            id: o.get_id(),
            obj: Some(obj.clone())
        }
    }

    /// Create a new DerivedRef with a dummy object.
    /// This is useful for initializing a DerivedRef with an object that is not yet parsed.
    /// Currently this is used exclusively in [super::GameState::get_vassal].
    pub fn dummy() -> Self{
        DerivedRef{
            id: 0,
            obj: None
        }
    }

    /// Initialize the DerivedRef with a [Shared] object.
    pub fn init(&mut self, obj:Shared<T>){
        self.id = obj.get_internal().get_id();
        self.obj = Some(obj);
    }

    pub fn get_ref(&self) -> Shared<T>{
        self.obj.as_ref().unwrap().clone()
    }
}

/// Converts an array of GameObjectDerived to an array of DerivedRef
pub fn serialize_array<T>(array:&Vec<Shared<T>>) -> Vec<DerivedRef<T>> where T:Renderable{
    let mut res = Vec::new();
    for s in array.iter(){
        res.push(DerivedRef::<T>::from_derived(s.clone()));
    }
    res
}

impl<T> Serialize for DerivedRef<T> where T:Renderable + Cullable{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let mut state = serializer.serialize_struct("DerivedRef", 4)?;
        state.serialize_field("id", &self.id)?;
        let o = self.obj.as_ref().unwrap().get_internal();
        state.serialize_field("name", &o.get_name())?;
        let shallow = o.get_depth() == 0;
        state.serialize_field("shallow", &shallow)?;
        state.serialize_field("subdir", T::get_subdir())?;
        state.end()
    }
}

impl<T> GameObjectDerived for DerivedRef<T> where T:Renderable + Cullable{
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> crate::game_object::GameString {
        self.obj.as_ref().unwrap().get_internal().get_name()
    }
}

impl<T> Cullable for DerivedRef<T> where T:Renderable + Cullable{
    fn get_depth(&self) -> usize {
        self.obj.as_ref().unwrap().get_internal().get_depth()
    }

    fn set_depth(&mut self, depth:usize, localization:&Localizer) {
        self.obj.as_ref().unwrap().get_internal_mut().set_depth(depth, localization);
    }
}

impl<T> Renderable for DerivedRef<T> where T:Renderable + Cullable{
    fn get_context(&self) -> minijinja::Value {
        self.obj.as_ref().unwrap().get_internal().get_context()
    }

    fn get_path(&self, path: &str) -> String {
        self.obj.as_ref().unwrap().get_internal().get_path(path)
    }

    fn get_subdir() -> &'static str {
        T::get_subdir()
    }

    fn get_template() -> &'static str {
        T::get_template()
    }

    fn render_all(&self, renderer: &mut Renderer, game_map: Option<&GameMap>, grapher: Option<&Grapher>) {
        self.obj.as_ref().unwrap().get_internal().render_all(renderer, game_map, grapher);
    }
}
