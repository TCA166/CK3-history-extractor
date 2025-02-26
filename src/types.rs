use std::{
    cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut},
    rc::Rc,
};

/// A type alias for a game object id.
pub type GameId = u32;

// implementing the Wrapper trait for GameId is overkill, the opaqueness is not needed as it's always going to be a numeric type

/// A type alias for a game string.
/// Roughly meant to represent a raw string from a save file, reference counted so that it exists once in memory.
/// Actually a [Rc] around a [str].
/// Comparisons might not work because compiler shenanigans, try [Rc::as_ref] when in doubt
pub type GameString = Rc<str>;

// Opaque type aliases for the standard library types.
/// A type alias for a hash map.
pub type HashMap<K, V> = std::collections::HashMap<K, V>;

/// A trait for objects that wrap a certain value.
/// Allows us to create opaque type aliases for certain types.
/// For example [GameString](crate::parser::GameString) is a wrapper around a reference counted string that implements this trait meaning if we wanted to change how the reference counting works we can do it with no interface changes.
/// Literally just a different flavor of [Deref].
pub trait Wrapper<T> {
    /// Wrap a value in the object
    fn wrap(t: T) -> Self;

    /// Get the internal value as a reference or a raw value
    fn get_internal(&self) -> Ref<T>;

    fn try_get_internal(&self) -> Result<Ref<T>, BorrowError>;
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
#[derive(Debug)]
pub struct Shared<T: ?Sized> {
    inner: Rc<RefCell<T>>,
}

impl<T> From<T> for Shared<T> {
    fn from(value: T) -> Self {
        Self::wrap(value)
    }
}

impl<T> Wrapper<T> for Shared<T> {
    fn wrap(t: T) -> Self {
        Shared {
            inner: Rc::new(RefCell::new(t)),
        }
    }

    fn get_internal(&self) -> Ref<T> {
        self.inner.borrow()
    }

    fn try_get_internal(&self) -> Result<Ref<T>, BorrowError> {
        self.inner.try_borrow()
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
