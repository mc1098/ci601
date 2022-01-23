//! Structs for representing a generic bibliographic entry and all its parts.
mod entry;
mod quoted_string;

use std::{borrow::Cow, collections::HashMap, marker::PhantomData};

pub use entry::*;
pub use quoted_string::{EscapePattern, QuotedString};

/// A [`Biblio`] builder used for managing a set of entry builders until they all succeed in order
/// to make a [`Biblio`] with valid entries in.
#[derive(Debug)]
pub struct BiblioBuilder {
    failed: bool,
    builders: Vec<Builder>,
    entries: Vec<Entry>,
}

impl BiblioBuilder {
    /// Attempts to build all of the entry builders and returns the [`Biblio`] if all of them
    /// succeed.
    ///
    /// # Errors
    ///
    /// Returns [`Err(Self)`] if one of the entry builders fail, this allows for resolving the
    /// entry builders that failed and then retrying the build.
    pub fn build(mut self) -> Result<Biblio, Self> {
        let (built, builders): (Vec<_>, Vec<_>) =
            try_partition(self.builders.into_iter().map(Builder::build));

        self.entries.extend(built);

        if builders.is_empty() {
            Ok(Biblio {
                dirty: self.failed,
                entries: self
                    .entries
                    .into_iter()
                    .map(|e| (e.cite().to_owned(), e))
                    .collect(),
            })
        } else {
            self.builders = builders;
            self.failed = true;
            Err(self)
        }
    }

    /// Returns the builders that failed to build so that missing fields can be set before trying
    /// to call [`BiblioBuilder::build`] again.
    pub fn unresolved(&mut self) -> impl Iterator<Item = &mut Builder> {
        self.builders.iter_mut()
    }

    /// Removes either the entry or builder based on the index.
    ///
    /// The [`BiblioBuilder`] can contain both resolvd entries or builders and does so in this
    /// order, therefore the index can be used to retrieve either.
    ///
    /// The index should be found using the [`BiblioBuilder::map_iter_all`] iterator as this
    /// iterator is in the same order.
    pub fn checked_remove(&mut self, index: usize) -> Option<Result<Entry, Builder>> {
        if index < self.entries.len() {
            Some(Ok(self.entries.remove(index)))
        } else if index - self.entries.len() < self.builders.len() {
            Some(Err(self.builders.remove(index - self.entries.len())))
        } else {
            None
        }
    }

    /// Returns an iterator of the result of the closure which is applied over both the resolve
    /// entries and unresolved builders.
    pub fn map_iter_all<F, T>(&self, f: F) -> MapIter<'_, F, T>
    where
        F: Fn(&dyn FieldQuery) -> T,
    {
        MapIter::new(self, f)
    }
}

/// Iterator of the resolved and unresolved entries a [`BiblioBuilder`] based on the result of a
/// closure given.
pub struct MapIter<'builder, F, T> {
    builder: &'builder BiblioBuilder,
    index: usize,
    f: F,
    _item: PhantomData<T>,
}

impl<'builder, F, T> MapIter<'builder, F, T>
where
    F: Fn(&dyn FieldQuery) -> T,
{
    fn new(builder: &'builder BiblioBuilder, f: F) -> Self {
        Self {
            builder,
            f,
            index: 0,
            _item: PhantomData,
        }
    }
}

impl<F, T> Iterator for MapIter<'_, F, T>
where
    F: Fn(&dyn FieldQuery) -> T,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let entries_size = self.builder.entries.len();

        if self.index < entries_size {
            let index = self.index;
            self.index += 1;
            self.builder.entries.get(index).map(|entry| (self.f)(entry))
        } else {
            let index = self.index - entries_size;
            self.index += 1;
            self.builder
                .builders
                .get(index)
                .map(|builder| (self.f)(builder))
        }
    }
}

fn try_partition<T, E, B, R>(iter: impl Iterator<Item = Result<T, E>>) -> (B, R)
where
    B: Default + Extend<T>,
    R: Default + Extend<E>,
{
    let mut left = B::default();
    let mut right = R::default();

    iter.fold((), |_, res| match res {
        Err(r) => right.extend([r]),
        l => left.extend(l),
    });

    (left, right)
}

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

    /// Attempts to build all of the entry builders and if they all succeed then returns a
    /// [`Biblio`].
    ///
    /// # Errors
    ///
    /// Returns [`Err(BiblioBuilder)`] if one of the entry builders fail, this allows resolving
    /// the builders and retrying the build.
    pub fn try_build(builders: Vec<Builder>) -> Result<Self, BiblioBuilder> {
        BiblioBuilder {
            failed: false,
            builders,
            entries: Vec::new(),
        }
        .build()
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
