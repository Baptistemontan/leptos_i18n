pub mod formatter;
pub mod key;

use std::fmt::Debug;

pub use key::{Key, KeyPath};

/// We should avoid to panic as much as possible, and return the Error enum instead,
/// but there is cases where unwrap *should* be good, like when accessing a value in a Map where the keys are already known
/// This trait serves as a easy unwrap where the code position can be given.
pub trait UnwrapAt {
    type Value;

    fn unwrap_at(self, location: &str) -> Self::Value;
}

impl<T> UnwrapAt for Option<T> {
    type Value = T;

    fn unwrap_at(self, location: &str) -> Self::Value {
        let msg = format!("Unexpected None value at {}. If you got this error please open an issue on the leptos_i18n github repo.", location);
        self.expect(&msg)
    }
}

impl<T, E: Debug> UnwrapAt for Result<T, E> {
    type Value = T;

    fn unwrap_at(self, location: &str) -> Self::Value {
        let msg = format!("Unexpected Err value at {}. If you got this error please open an issue on the leptos_i18n github repo.", location);
        self.expect(&msg)
    }
}
