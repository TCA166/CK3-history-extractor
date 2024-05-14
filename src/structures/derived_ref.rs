use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::game_object::{GameString, WrapperMut};

use super::{Cullable, GameId, GameObjectDerived, Shared, Wrapper};

/// A shallow serializable reference to a derived game object.
/// The idea is to provide the id and name of the object, without serializing the whole object.
/// This is useful for serializing references to objects that are not in the current scope.
pub struct DerivedRef<T> where T:GameObjectDerived + Cullable{
    id: GameId,
    name: GameString,
    obj: Shared<T>
}

impl<T> DerivedRef<T> where T:GameObjectDerived + Cullable{
    /// Create a new DerivedRef from a [Shared] object.
    /// This will clone the object and store a reference to it.
    pub fn from_derived(obj:Shared<T>) -> Self{
        let o = obj.get_internal();
        DerivedRef{
            id: o.get_id(),
            name: o.get_name(),
            obj: obj.clone()
        }
    }

    /// Create a new DerivedRef with a dummy object.
    /// This is useful for initializing a DerivedRef with an object that is not yet parsed.
    /// Currently this is used exclusively in [super::GameState::get_vassal].
    pub fn dummy() -> Self{
        DerivedRef{
            id: 0,
            name: GameString::wrap("".to_string()),
            obj: Shared::wrap(T::dummy(0))
        }
    }

    /// Initialize the DerivedRef with a [Shared] object.
    pub fn init(&mut self, obj:Shared<T>){
        self.id = obj.get_internal().get_id();
        self.name = obj.get_internal().get_name();
        self.obj = obj;
    }

    pub fn get_ref(&self) -> Shared<T>{
        self.obj.clone()
    }
}

/// Converts an array of GameObjectDerived to an array of DerivedRef
pub fn serialize_array<T>(array:&Vec<Shared<T>>) -> Vec<DerivedRef<T>> where T:GameObjectDerived + Cullable{
    let mut res = Vec::new();
    for s in array.iter(){
        res.push(DerivedRef::<T>::from_derived(s.clone()));
    }
    res
}

impl<T> Serialize for DerivedRef<T> where T:GameObjectDerived + Cullable{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let mut state = serializer.serialize_struct("DerivedRef", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        let shallow = self.obj.get_internal().get_depth() == 0;
        state.serialize_field("shallow", &shallow)?;
        state.end()
    }
}

impl<T> Cullable for DerivedRef<T> where T:GameObjectDerived + Cullable{
    fn get_depth(&self) -> usize {
        self.obj.get_internal().get_depth()
    }

    fn set_depth(&mut self, depth:usize) {
        self.obj.get_internal_mut().set_depth(depth);
    }
}
