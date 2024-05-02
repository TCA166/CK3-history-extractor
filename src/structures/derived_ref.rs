use serde::Serialize;
use serde::ser::SerializeStruct;

use super::{Cullable, GameObjectDerived, Shared};

struct DerivedRef<T> where T:GameObjectDerived + Cullable{
    pub id: u32,
    pub name: Shared<String>,
    obj: T
}

impl<T> DerivedRef<T> where T:GameObjectDerived + Cullable{
    pub fn new(id:u32, name:Shared<String>, obj:T) -> Self{
        DerivedRef{
            id,
            name,
            obj
        }
    }

    pub fn from_derived(obj:T) -> Self{
        DerivedRef{
            id: obj.get_id(),
            name: obj.get_name(),
            obj
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
        self.obj.get_depth()
    }

    fn set_depth(&mut self, depth:usize) {
        self.obj.set_depth(depth);
    }
}
