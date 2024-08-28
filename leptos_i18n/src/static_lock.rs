#![doc(hidden)]
use std::sync::{OnceLock, RwLock};

#[derive(Debug, Default)]
pub struct StaticLock<T>(OnceLock<RwLock<T>>);

impl<T> StaticLock<T> {
    pub const fn new() -> Self {
        StaticLock(OnceLock::new())
    }

    pub fn with_mut<U>(&self, f: impl FnOnce(&mut T) -> U) -> U
    where
        T: Default,
    {
        let mutex = self.0.get_or_init(Default::default);
        let mut guard = mutex.write().unwrap();
        f(&mut guard)
    }

    pub fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U
    where
        T: Default,
    {
        let mutex = self.0.get_or_init(Default::default);
        let guard = mutex.read().unwrap();
        f(&guard)
    }
}

#[derive(Debug, Default)]
pub struct StaticLockOnce<T: 'static>(OnceLock<&'static T>);

fn leak_default<T: Default>() -> &'static T {
    Box::leak(Box::default())
}

impl<T: 'static> StaticLockOnce<T> {
    pub const fn new() -> Self {
        StaticLockOnce(OnceLock::new())
    }

    pub fn get(&self) -> Option<&'static T> {
        self.0.get().copied()
    }

    pub fn get_or_default(&self) -> &'static T
    where
        T: Default,
    {
        self.0.get_or_init(leak_default)
    }

    pub fn get_or_init(&self, f: impl FnOnce() -> T) -> &'static T {
        self.0.get_or_init(|| Box::leak(Box::new(f())))
    }
}
