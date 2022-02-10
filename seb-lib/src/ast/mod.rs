//! Structs for representing a generic bibliographic entry and all its parts.
mod biblio;
mod entry;
mod quoted_string;

use std::borrow::Cow;

pub use biblio::*;
pub use entry::*;
pub use quoted_string::{EscapePattern, QuotedString};

/// An entry field which is essentially a key value pair.
#[derive(Clone, Debug, PartialEq)]
pub struct Field<'name, 'value> {
    /// Name of the entry field.
    pub name: Cow<'name, str>,
    /// Value of the entry field.
    pub value: Cow<'value, QuotedString>,
}

impl Field<'_, '_> {
    /// The `&str` representation of the `value` field.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}
