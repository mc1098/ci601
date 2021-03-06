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
pub struct Field<'entry> {
    /// Name of the entry field.
    pub name: Cow<'entry, str>,
    /// Value of the entry field.
    pub value: Cow<'entry, QuotedString>,
}

impl Field<'_> {
    /// The `&str` representation of the `value` field.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl<'entry> From<(&'entry String, &'entry QuotedString)> for Field<'entry> {
    fn from((k, v): (&'entry String, &'entry QuotedString)) -> Self {
        Self {
            name: Cow::Borrowed(k),
            value: Cow::Borrowed(v),
        }
    }
}

impl<'entry> From<(&'entry str, &'entry QuotedString)> for Field<'entry> {
    fn from((k, v): (&'entry str, &'entry QuotedString)) -> Self {
        Self {
            name: Cow::Borrowed(k),
            value: Cow::Borrowed(v),
        }
    }
}
