use serde::Serialize;
use serde::ser::SerializeStruct;

use super::{Cullable, GameObjectDerived, Shared};

/// A shallow serializable reference to a derived game object.
/// The idea is to provide the id and name of the object, without serializing the whole object.
/// This is useful for serializing references to objects that are not in the current scope.
pub struct DerivedRef<T> where T:GameObjectDerived + Cullable{
    pub id: u32,
    pub name: Shared<String>,
    obj: Shared<T>
}

impl<T> DerivedRef<T> where T:GameObjectDerived + Cullable{
    /// Create a new DerivedRef from a [Shared] object.
    /// This will clone the object and store a reference to it.
    pub fn from_derived(obj:Shared<T>) -> Self{
        let o = obj.borrow();
        DerivedRef{
            id: o.get_id(),
            name: o.get_name(),
            obj: obj.clone()
        }
    }
}

impl<T> Serialize for DerivedRef<T> where T:GameObjectDerived + Cullable{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let mut state = serializer.serialize_struct("DerivedRef", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.end()
    }
}

impl<T> Cullable for DerivedRef<T> where T:GameObjectDerived + Cullable{
    fn get_depth(&self) -> usize {
        self.obj.borrow().get_depth()
    }

    fn set_depth(&mut self, depth:usize) {
        self.obj.borrow_mut().set_depth(depth);
    }
}
