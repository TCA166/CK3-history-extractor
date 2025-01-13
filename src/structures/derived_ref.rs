use std::ops::{Deref, DerefMut};

use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};

use super::{
    super::{
        display::{Cullable, Grapher, Renderable, RenderableType},
        game_data::GameData,
        parser::{GameState, GameString},
        types::WrapperMut,
    },
    GameId, GameObjectDerived, Shared, Wrapper,
};

/// A shallow serializable reference to a derived game object.
/// The idea is to provide the id and name of the object, without serializing the whole object.
/// This is useful for serializing references to objects that are not in the current scope.
pub struct DerivedRef<T>
where
    T: GameObjectDerived,
{
    id: GameId,
    obj: Option<Shared<T>>,
}

impl<T> DerivedRef<T>
where
    T: GameObjectDerived,
{
    /// Create a new DerivedRef with a dummy object.
    /// This is useful for initializing a DerivedRef with an object that is not yet parsed.
    /// Currently this is used exclusively in [super::GameState::get_vassal].
    pub fn dummy() -> Self {
        DerivedRef { id: 0, obj: None }
    }

    /// Initialize the DerivedRef with a [Shared] object.
    pub fn init(&mut self, obj: Shared<T>) {
        self.id = obj.get_internal().get_id();
        self.obj = Some(obj);
    }

    pub fn get_ref(&self) -> Shared<T> {
        self.obj.as_ref().unwrap().clone()
    }
}

/// Converts an array of GameObjectDerived to an array of DerivedRef
pub fn into_ref_array<T>(array: &Vec<Shared<T>>) -> Vec<DerivedRef<T>>
where
    T: Renderable,
{
    let mut res = Vec::new();
    for s in array.iter() {
        res.push(DerivedRef::<T>::from(s.clone()));
    }
    res
}

/// Serialize a [Shared] object as a [DerivedRef].
pub fn serialize_ref<S: Serializer, T: Renderable>(
    value: &Option<Shared<T>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match value {
        Some(v) => {
            let derived = DerivedRef::<T>::from(v.clone());
            derived.serialize(serializer)
        }
        None => serializer.serialize_none(),
    }
}

/// Serialize a [Vec] of [Shared] objects as a [Vec] of [DerivedRef] objects.
pub fn serialize_array_ref<S: Serializer, T: Renderable>(
    value: &Vec<Shared<T>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut state = serializer.serialize_seq(Some(value.len()))?;
    for v in value.iter() {
        let derived = DerivedRef::<T>::from(v.clone());
        state.serialize_element(&derived)?;
    }
    state.end()
}

impl<T> Serialize for DerivedRef<T>
where
    T: Renderable,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
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

impl<T> GameObjectDerived for DerivedRef<T>
where
    T: Renderable,
{
    fn get_id(&self) -> GameId {
        self.id
    }

    fn get_name(&self) -> GameString {
        self.obj.as_ref().unwrap().get_internal().get_name()
    }
}

impl<T> Cullable for DerivedRef<T>
where
    T: Renderable,
{
    fn get_depth(&self) -> usize {
        self.obj.as_ref().unwrap().get_internal().get_depth()
    }

    fn set_depth(&mut self, depth: usize) {
        self.obj
            .as_ref()
            .unwrap()
            .get_internal_mut()
            .set_depth(depth);
    }
}

impl<T> Renderable for DerivedRef<T>
where
    T: Renderable,
{
    fn get_path(&self, path: &str) -> String {
        self.obj.as_ref().unwrap().get_internal().get_path(path)
    }

    fn get_subdir() -> &'static str {
        T::get_subdir()
    }

    fn get_template() -> &'static str {
        T::get_template()
    }

    fn append_ref(&self, stack: &mut Vec<RenderableType>) {
        self.obj.as_ref().unwrap().get_internal().append_ref(stack);
    }

    fn render(
        &self,
        path: &str,
        game_state: &GameState,
        grapher: Option<&Grapher>,
        data: &GameData,
    ) {
        self.obj
            .as_ref()
            .unwrap()
            .get_internal()
            .render(path, game_state, grapher, data);
    }
}

impl<T> Deref for DerivedRef<T>
where
    T: GameObjectDerived,
{
    type Target = Shared<T>;

    fn deref(&self) -> &Self::Target {
        self.obj.as_ref().unwrap()
    }
}

impl<T> DerefMut for DerivedRef<T>
where
    T: GameObjectDerived,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.obj.as_mut().unwrap()
    }
}

impl<T> From<Shared<T>> for DerivedRef<T>
where
    T: GameObjectDerived,
{
    fn from(obj: Shared<T>) -> Self {
        let o = obj.get_internal();
        DerivedRef {
            id: o.get_id(),
            obj: Some(obj.clone()),
        }
    }
}

impl<T> TryInto<Shared<T>> for DerivedRef<T>
where
    T: GameObjectDerived,
{
    type Error = &'static str;

    fn try_into(self) -> Result<Shared<T>, Self::Error> {
        self.obj.ok_or("Object not initialized")
    }
}
