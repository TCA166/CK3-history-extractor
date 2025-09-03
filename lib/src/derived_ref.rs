use serde::Serialize;

use super::{
    structures::{FromGameObject, GameObjectDerived, GameRef},
    types::Wrapper,
};

pub const DERIVED_REF_NAME_ATTR: &str = "name";
pub const DERIVED_REF_SUBDIR_ATTR: &str = "subdir";
pub const DERIVED_REF_ID_ATTR: &str = "id";

#[cfg(feature = "display")]
mod display {
    use super::super::display::ProceduralPath;
    use super::*;

    use serde::ser::SerializeStruct;

    impl<T: GameObjectDerived + FromGameObject + ProceduralPath> Serialize for GameRef<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let internal = self.get_internal();
            if let Some(inner) = internal.inner() {
                let mut state = serializer.serialize_struct("DerivedRef", 3)?;
                state.serialize_field(DERIVED_REF_ID_ATTR, &internal.get_id())?;
                state.serialize_field(DERIVED_REF_NAME_ATTR, &inner.get_name())?;
                state.serialize_field(DERIVED_REF_SUBDIR_ATTR, T::SUBDIR)?;
                state.end()
            } else {
                serializer.serialize_none()
            }
        }
    }
}

#[cfg(not(feature = "display"))]
mod not_display {
    use super::*;

    impl<T: GameObjectDerived + FromGameObject + Serialize> Serialize for GameRef<T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.get_internal().serialize(serializer)
        }
    }
}
