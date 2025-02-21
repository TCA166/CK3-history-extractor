use std::{
    cell::{BorrowMutError, Ref, RefCell, RefMut},
    ops::Deref,
    rc::Rc,
};

use super::structures::GameObjectDerived;

use serde::Serialize;

// Opaque type aliases for the standard library types.
/// A type alias for a hash map.
pub type HashMap<K, V> = std::collections::HashMap<K, V>;
/// A type alias for a hash set.
pub type HashSet<T> = std::collections::HashSet<T>;

/// A reference or a raw value. I have no clue why this isn't a standard library type.
/// A [Ref] and a raw reference are both dereferencable to the same type.
pub enum RefOrRaw<'a, T: 'a> {
    Ref(Ref<'a, T>),
    Raw(&'a T),
}

impl<T: GameObjectDerived> PartialEq for RefOrRaw<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl<'a, T> Deref for RefOrRaw<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            RefOrRaw::Ref(r) => r.deref(),
            RefOrRaw::Raw(r) => r,
        }
    }
}

impl<'a, T: Serialize> Serialize for RefOrRaw<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RefOrRaw::Ref(r) => r.deref().serialize(serializer),
            RefOrRaw::Raw(r) => r.serialize(serializer),
        }
    }
}

/// A trait for objects that wrap a certain value.
/// Allows us to create opaque type aliases for certain types.
/// For example [GameString](crate::parser::GameString) is a wrapper around a reference counted string that implements this trait meaning if we wanted to change how the reference counting works we can do it with no interface changes.
/// Literally just a different flavor of [Deref].
pub trait Wrapper<T> {
    /// Wrap a value in the object
    fn wrap(t: T) -> Self;

    /// Get the internal value as a reference or a raw value
    fn get_internal(&self) -> RefOrRaw<T>;
}

/// A trait for objects that wrap a certain value and allow mutation.
/// Allows us to create opaque type aliases for certain types.
/// Literally just a different flavor of [std::ops::DerefMut].
pub trait WrapperMut<T> {
    /// Get the internal value as a mutable reference
    fn get_internal_mut(&self) -> RefMut<T>;

    /// Try to get the internal value as a mutable reference
    fn try_get_internal_mut(&self) -> Result<RefMut<T>, BorrowMutError>;
}

/// A type alias for shared objects.
///
/// # Example
///
/// ```
/// let obj:Shared<String> = Shared::wrap("Hello");
///
/// let value:Ref<String> = obj.get_internal();
/// ```
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct Shared<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> Wrapper<T> for Shared<T> {
    fn wrap(t: T) -> Self {
        Shared {
            inner: Rc::new(RefCell::new(t)),
        }
    }

    fn get_internal(&self) -> RefOrRaw<T> {
        RefOrRaw::Ref(self.inner.borrow())
    }
}

impl<T> WrapperMut<T> for Shared<T> {
    fn get_internal_mut(&self) -> RefMut<T> {
        self.inner.borrow_mut()
    }

    fn try_get_internal_mut(&self) -> Result<RefMut<T>, BorrowMutError> {
        self.inner.try_borrow_mut()
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared {
            inner: self.inner.clone(),
        }
    }
}

impl<T: GameObjectDerived + Serialize> Serialize for Shared<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get_internal().serialize(serializer)
    }
}

/// A type that can be either a single value or a vector of values.
pub enum OneOrMany<'a, T> {
    One(&'a Shared<T>),
    Many(&'a Vec<Shared<T>>),
}
