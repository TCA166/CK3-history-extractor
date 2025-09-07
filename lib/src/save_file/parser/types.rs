use std::{
    cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut},
    error,
    fmt::Debug,
    rc::Rc,
};

use derive_more::{Display, From};
use jomini::{
    binary::{ReaderError as BinaryReaderError, Token as BinaryToken},
    text::{ReaderError as TextReaderError, Token as TextToken},
};

/// An error that can occur when reading from a tape.
#[derive(Debug, From, Display)]
pub enum TapeError {
    Text(TextReaderError),
    Binary(BinaryReaderError),
}

impl error::Error for TapeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Text(err) => Some(err),
            Self::Binary(err) => Some(err),
        }
    }
}

/* We only have this rather opaque abstraction, not a generalization of tokens
because a generalization would discard too much context. Most notably, the
information regarding whether a string was quoted or not. As such, we only
have this abstraction, used exclusively in error handling. */

/// An abstraction over [jomini] tokens: [jomini::TextToken] and [jomini::BinaryToken]
#[derive(From, Debug)]
pub enum Token<'a> {
    Text(TextToken<'a>),
    Binary(BinaryToken<'a>),
}

/// A type alias for a game object id.
pub type GameId = u32;

// implementing the Wrapper trait for GameId is overkill, the opaqueness is not needed as it's always going to be a numeric type

/// A type alias for a game string.
/// Roughly meant to represent a raw string from a save file, reference counted so that it exists once in memory.
/// Actually a [Rc] around a [str].
/// Comparisons might not work because compiler shenanigans, try [Rc::as_ref] when in doubt
pub type GameString = Rc<str>;

/// A trait for objects that wrap a certain value.
/// Allows us to create opaque type aliases for certain types.
pub trait Wrapper<T> {
    /// Wrap a value in the object
    fn wrap(t: T) -> Self;

    /// Get the internal value as a reference or a raw value. Will panic if the value is already mutably borrowed.
    fn get_internal(&self) -> Ref<T>;

    /// Try to get the internal value as a reference or a raw value
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

/// Really just a type alias for [Rc]<[RefCell]>
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
