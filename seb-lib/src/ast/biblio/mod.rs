use std::collections::HashMap;

mod resolver;

pub use resolver::BiblioResolver;

use super::{Entry, FieldQuery, QuotedString, Resolver};

/// An intermediate representation of a bibliography which is not tied to a specific end format.
#[derive(Debug, Default, PartialEq)]
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
            .any(|e| e.get_field(key).map(&predicate).unwrap_or_default())
    }

    /// Returns the reference to the `Entry` corresponding to the cite key value.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Entry> {
        self.entries.get(key)
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::ast::Manual;

    use super::*;

    #[test]
    fn insertion_of_entry_makes_biblio_dirty() {
        let mut biblio = Biblio::default();

        assert!(!biblio.dirty(), "Biblio::default should be clean");

        biblio.insert(Entry::Manual(Manual {
            cite: "cite".to_owned(),
            title: "Title".into(),
            optional: HashMap::default(),
        }));

        assert!(
            biblio.dirty(),
            "After insertion of an entry the Biblio should be dirty"
        );
        assert!(
            !biblio.dirty(),
            "After calling Biblio::dirty the flag is reset so this second call \
                to the function should return false"
        );
    }

    #[test]
    fn dirty_flag_should_not_be_effected_when_nothing_is_removed() {
        let mut biblio = Biblio::default();

        assert!(
            !biblio.remove("this doesn't exist!"),
            "The Biblio is empty so nothing can be removed"
        );
        assert!(
            !biblio.dirty(),
            "Nothing was removed so the dirty flag should still be false"
        );
    }

    #[test]
    fn remove_entry_in_single_biblio() {
        let mut biblio = Biblio::new(vec![Entry::Manual(Manual {
            cite: "cite".to_owned(),
            title: "Title".into(),
            optional: HashMap::default(),
        })]);

        assert!(biblio.remove("cite"), "Should remove the only entry");
        assert!(biblio.dirty());
        assert!(
            biblio.into_entries().is_empty(),
            "The only entry should have been removed"
        );
    }

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
