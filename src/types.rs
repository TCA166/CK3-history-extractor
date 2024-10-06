use std::{
    cell::{BorrowMutError, Ref, RefCell, RefMut},
    ops::Deref,
    rc::Rc,
};

/// A reference or a raw value. I have no clue why this isn't a standard library type.
/// A [Ref] and a raw reference are both dereferencable to the same type.
pub enum RefOrRaw<'a, T: 'a> {
    Ref(Ref<'a, T>),
    Raw(&'a T),
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

/// A trait for objects that wrap a certain value.
/// Allows us to create opaque type aliases for certain types.
/// For example [GameString](crate::parser::GameString) is a wrapper around a reference counted string that implements this trait meaning if we wanted to change how the reference counting works we can do it with no interface changes.
pub trait Wrapper<T> {
    /// Wrap a value in the object
    fn wrap(t: T) -> Self;

    /// Get the internal value as a reference or a raw value
    fn get_internal(&self) -> RefOrRaw<T>;
}

/// A trait for objects that wrap a certain value and allow mutation.
/// Allows us to create opaque type aliases for certain types.
pub trait WrapperMut<T> {
    /// Get the internal value as a mutable reference
    fn get_internal_mut(&self) -> RefMut<T>;

    /// Try to get the internal value as a mutable reference
    fn try_get_internal_mut(&self) -> Result<RefMut<T>, BorrowMutError>;
}

/// A type alias for shared objects.
/// Aliases: [std::rc::Rc]<[std::cell::RefCell]<>>
///
/// # Example
///
/// ```
/// let obj:Shared<String> = Shared::wrap("Hello");
///
/// let value:Ref<String> = obj.get_internal();
/// ```
pub type Shared<T> = Rc<RefCell<T>>;

impl<T> Wrapper<T> for Shared<T> {
    fn wrap(t: T) -> Self {
        Rc::new(RefCell::new(t))
    }

    fn get_internal(&self) -> RefOrRaw<T> {
        RefOrRaw::Ref(self.borrow())
    }
}

impl<T> WrapperMut<T> for Shared<T> {
    fn get_internal_mut(&self) -> RefMut<T> {
        self.borrow_mut()
    }

    fn try_get_internal_mut(&self) -> Result<RefMut<T>, BorrowMutError> {
        self.try_borrow_mut()
    }
}
