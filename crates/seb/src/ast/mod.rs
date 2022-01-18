//! Structs for representing a generic bibliographic entry and all its parts.
mod entry;
mod quoted_string;

use std::{borrow::Cow, collections::HashMap};

pub use entry::*;
pub use quoted_string::{EscapePattern, QuotedString};

/// An intermediate representation of a bibliography which is not tied to a specific end format.
#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Clone))]
pub struct Biblio {
    dirty: bool,
    entries: HashMap<String, Entry>,
}

impl Biblio {
    /// Create a new [`Biblio`] from a list of bibliography entries.
    #[must_use]
    pub fn new(entries: Vec<Entry>) -> Self {
        Self {
            dirty: false,
            entries: entries
                .into_iter()
                .map(|e| (e.cite().to_owned(), e))
                .collect(),
        }
    }

    /// Checks and resets the `dirty` flag.
    ///
    /// The `dirty` flag will return true when this instance has been edited since it was created.
    /// The default value of the `dirty` flag is `false`, therefore calling this function will
    /// always reset the `dirty` flag to `false`.
    pub fn dirty(&mut self) -> bool {
        let dirty = self.dirty;
        self.dirty = false;
        dirty
    }

    /// Insert a new [`Entry`].
    pub fn insert(&mut self, entry: Entry) {
        self.dirty = true;
        self.entries.insert(entry.cite().to_owned(), entry);
    }

    /// Remove the cite key and return the [`Entry`] value if they cite was found.
    pub fn remove(&mut self, cite: &str) -> Option<Entry> {
        let entry = self.entries.remove(cite);
        self.dirty |= entry.is_some();
        entry
    }

    /// Return a reference to a slice of entries.
    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.values()
    }

    /// Creates entries from a value.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // drop is not const
    pub fn into_entries(self) -> Vec<Entry> {
        self.entries.into_iter().map(|(_, v)| v).collect()
    }

    /// Tests if any field in this [`Biblio`] matches a predicate.
    ///
    /// [`Self::contains_field`] takes a `key` value that should match a [`Field::name`] and
    /// [`Self::contains_field`] takes a closure that returns `true` or `false`. It applies this
    /// closure to each field in each entry of the [`Biblio`], and if any of them return `true`, then
    /// so does [`Self::contains_field`]. If they all return `false`, it returns `false`.
    ///
    /// [`Self::contains_field`] is short-circuiting; in other words, it will stop processing as
    /// soon as it finds a `true`, given that no matter what else happens, the result will also be
    /// `true`.
    ///
    /// An empty [`Biblio`] will always return `false`.
    pub fn contains_field<P>(&self, key: &str, predicate: P) -> bool
    where
        P: Fn(&QuotedString) -> bool,
    {
        self.entries
            .values()
            .any(|e| e.find_field(key).map(&predicate).unwrap_or_default())
    }

    /// Merges one [`Biblio`] into another and replaces any existing entries with the same cite key
    /// with the ones being merged in.
    pub fn merge(&mut self, other: Biblio) {
        self.entries.extend(other.entries);
    }
}

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

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use super::*;

    #[test]
    fn false_on_duplicate_field() {
        let square_quote = |c: char| matches!(c, '{' | '}');
        let title = QuotedString::from_quoted(
            "{Quicksort}: A Fast Sorting Scheme in Theory and Practice",
            &square_quote,
        );
        let value = QuotedString::new("test".to_owned());
        let mut optional = HashMap::new();
        optional.insert("doi".to_owned(), value);
        let entry = Entry::Manual(Manual {
            cite: "Edelkamp_2019".to_owned(),
            title,
            optional,
        });
        let references = Biblio::new(vec![entry]);

        assert!(references.contains_field("doi", |f| &**f == "test"));
        assert!(!references.contains_field("doi", |f| &**f == "something else"));
    }
}
