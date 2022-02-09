use std::{collections::HashMap, marker::PhantomData};

use super::{Entry, FieldQuery, QuotedString, Resolver};

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

    /// Attempts to resolve all of the entry resolvers and if they all succeed then returns a
    /// [`Biblio`].
    ///
    /// # Errors
    ///
    /// Returns [`Err(BiblioResolver)`] if one of the entry resolvers fail, this allows resolving
    /// the resolvers and retrying the resolve.
    pub fn try_resolve(resolvers: Vec<Resolver>) -> Result<Self, BiblioResolver> {
        BiblioResolver {
            failed: false,
            resolvers,
            entries: Vec::new(),
        }
        .resolve()
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
    pub fn remove(&mut self, cite: &str) -> bool {
        let mut removed = false;
        self.entries.retain(|k, _| {
            let check = k.to_lowercase() != cite.to_lowercase();
            removed |= !check;
            check
        });

        self.dirty |= removed;
        removed
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
    /// [`Self::contains_field`] takes a `key` value that should match a [`Field::name`][FN] and
    /// [`Self::contains_field`] takes a closure that returns `true` or `false`. It applies this
    /// closure to each field in each entry of the [`Biblio`], and if any of them return `true`, then
    /// so does [`Self::contains_field`]. If they all return `false`, it returns `false`.
    ///
    /// [`Self::contains_field`] is short-circuiting; in other words, it will stop processing as
    /// soon as it finds a `true`, given that no matter what else happens, the result will also be
    /// `true`.
    ///
    /// An empty [`Biblio`] will always return `false`.
    ///
    /// [FN]: [seb::ast::Field::name]
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

/// A [`Biblio`] resolver used for managing a set of entry resolvers until they all succeed in order
/// to make a [`Biblio`] with valid entries in.
#[derive(Debug)]
pub struct BiblioResolver {
    failed: bool,
    resolvers: Vec<Resolver>,
    entries: Vec<Entry>,
}

impl BiblioResolver {
    /// Attempts to resolve all of the entry resolvers and returns the [`Biblio`] if all of them
    /// succeed.
    ///
    /// # Errors
    ///
    /// Returns [`Err(Self)`] if one of the entry resolvers fail, this allows for resolving the
    /// entry resolvers that failed and then retrying the resolve.
    pub fn resolve(mut self) -> Result<Biblio, Self> {
        let (built, resolvers): (Vec<_>, Vec<_>) =
            try_partition(self.resolvers.into_iter().map(Resolver::resolve));

        self.entries.extend(built);

        if resolvers.is_empty() {
            Ok(Biblio {
                dirty: self.failed,
                entries: self
                    .entries
                    .into_iter()
                    .map(|e| (e.cite().to_owned(), e))
                    .collect(),
            })
        } else {
            self.resolvers = resolvers;
            self.failed = true;
            Err(self)
        }
    }

    /// Returns the resolvers that failed to resolve so that missing fields can be set before trying
    /// to call [`BiblioResolver::resolve`] again.
    pub fn unresolved(&mut self) -> impl Iterator<Item = &mut Resolver> {
        self.resolvers.iter_mut()
    }

    /// Removes either the entry or resolver based on the index.
    ///
    /// The [`BiblioResolver`] can contain both resolvd entries or resolvers and does so in this
    /// order, therefore the index can be used to retrieve either.
    ///
    /// The index should be found using the [`BiblioResolver::map_iter_all`] iterator as this
    /// iterator is in the same order.
    pub fn checked_remove(&mut self, index: usize) -> Option<Result<Entry, Resolver>> {
        if index < self.entries.len() {
            Some(Ok(self.entries.remove(index)))
        } else if index - self.entries.len() < self.resolvers.len() {
            Some(Err(self.resolvers.remove(index - self.entries.len())))
        } else {
            None
        }
    }

    /// Returns an iterator of the result of the closure which is applied over both the resolve
    /// entries and unresolved resolvers.
    pub fn map_iter_all<F, T>(&self, f: F) -> MapIter<'_, F, T>
    where
        F: Fn(&dyn FieldQuery) -> T,
    {
        MapIter::new(self, f)
    }
}

/// Iterator of the resolved and unresolved entries a [`BiblioResolver`] based on the result of a
/// closure given.
pub struct MapIter<'resolver, F, T> {
    resolver: &'resolver BiblioResolver,
    index: usize,
    f: F,
    _item: PhantomData<T>,
}

impl<'resolver, F, T> MapIter<'resolver, F, T>
where
    F: Fn(&dyn FieldQuery) -> T,
{
    fn new(resolver: &'resolver BiblioResolver, f: F) -> Self {
        Self {
            resolver,
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
        let entries_size = self.resolver.entries.len();

        if self.index < entries_size {
            let index = self.index;
            self.index += 1;
            self.resolver
                .entries
                .get(index)
                .map(|entry| (self.f)(entry))
        } else {
            let index = self.index - entries_size;
            self.index += 1;
            self.resolver
                .resolvers
                .get(index)
                .map(|resolver| (self.f)(resolver))
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

impl std::fmt::Display for BiblioResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for unresolved in &self.resolvers {
            writeln!(f, "{}", unresolved)?;
        }

        write!(
            f,
            "hint: consider enabling interactive mode (-i / --interact) to add missing fields."
        )?;
        Ok(())
    }
}
impl std::error::Error for BiblioResolver {}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::ast::Manual;

    use super::*;

    #[test]
    fn false_on_duplicate_field() {
        let square_quote = |c: char| matches!(c, '{' | '}');
        let title = QuotedString::from_quoted(
            "{Quicksort}: A Fast Sorting Scheme in Theory and Practice",
            &square_quote,
        );
        let value = "test".into();
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
