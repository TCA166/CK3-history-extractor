use std::{ops::Deref, sync::{Arc, Mutex, MutexGuard, TryLockError}};

/// A reference or a raw value. I have no clue why this isn't a standard library type.
/// A [Ref] and a raw reference are both dereferencable to the same type.
pub enum RefOrRaw<'a, T: 'a> {
    Ref(MutexGuard<'a, T>),
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
/// For example [GameString] is a wrapper around a reference counted string that implements this trait meaning if we wanted to change how the reference counting works we can do it with no interface changes.
pub trait Wrapper<T> {
    /// Wrap a value in the object
    fn wrap(t:T) -> Self;

    fn get_internal(&self) -> RefOrRaw<T>;

    fn try_get_internal(&self) -> Result<RefOrRaw<T>, TryLockError<MutexGuard<T>>>;
}

/// A trait for objects that wrap a certain value and allow mutation.
/// Allows us to create opaque type aliases for certain types.
pub trait WrapperMut<T> {
    fn get_internal_mut(&self) -> MutexGuard<T>;

    fn try_get_internal_mut(&self) -> Result<MutexGuard<T>, TryLockError<MutexGuard<T>>>;
}

/// A type alias for shared objects.
/// Aliases: [std::rc::GameString]<[std::cell::RefCell]<>>
/// 
/// # Example
/// 
/// ```
/// let obj:Shared<String> = Shared::wrap("Hello");
/// 
/// let value:Ref<String> = obj.get_internal();
/// ```
pub type Shared<T> = Arc<Mutex<T>>;

impl<T> Wrapper<T> for Shared<T> {
    fn wrap(t:T) -> Self {
        Arc::new(Mutex::new(t))
    }

    fn get_internal(&self) -> RefOrRaw<T> {
        RefOrRaw::Ref(self.lock().unwrap())
    }

    fn try_get_internal(&self) -> Result<RefOrRaw<T>, TryLockError<MutexGuard<T>>> {
        let r = self.try_lock();
        match r {
            Ok(r) => Ok(RefOrRaw::Ref(r)),
            Err(e) => Err(e) // Fix: Provide the missing argument 'e' in the Err() variant
        }
    }
}

impl<T> WrapperMut<T> for Shared<T> {
    fn get_internal_mut(&self) -> MutexGuard<T> {
        self.lock().unwrap()
    }

    fn try_get_internal_mut(&self) -> Result<MutexGuard<T>, TryLockError<MutexGuard<T>>> {
        self.try_lock()
    }
}
