/// An intermediate representation of a bibliography which is not tied to a specific end format.
#[derive(Clone, Debug, PartialEq)]
pub struct Biblio(Vec<Entry>);

impl Biblio {
    /// Create a new [`Biblio`] from a list of bibliography entries.
    #[must_use]
    pub fn new(entries: Vec<Entry>) -> Self {
        Self(entries)
    }

    /// Insert a new [`Entry`].
    pub fn insert(&mut self, entry: Entry) {
        self.0.push(entry);
    }

    /// Return a reference to a slice of entries.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.0
    }

    /// Creates entries from a value.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // drop is not const
    pub fn into_entries(self) -> Vec<Entry> {
        self.0
    }

    /// Tests if any field in this [`Biblio`] matches a predicate.
    ///
    /// [`Self::contains_field`] takes a `key` value that should match a [`Field::Name`] and
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
        P: Fn(&Field) -> bool,
    {
        // Use find -> map so that find can short-circuit when reaching the correct field key
        // and then map will only every be applied to one field. If we used both predicates in
        // the `find` closure then we'd scan all the entries even if we found the correct key
        // first.
        self.0.iter().any(|e| {
            e.fields
                .iter()
                .find(|f| f.name == key)
                .map(&predicate)
                .unwrap_or_default()
        })
    }

    /// Sorts the entries and their fields.
    ///
    /// This is a function for use in tests in order to sort items in a [`Vec`] before
    /// a equality check is performed.
    #[cfg(test)]
    pub fn sort_entries(&mut self) {
        for entry in &mut self.0 {
            entry.fields.sort_by(|a, b| a.name.cmp(&b.name));
        }

        self.0.sort_by(|a, b| a.cite.cmp(&b.cite));
    }
}

impl IntoIterator for Biblio {
    type Item = Entry;

    type IntoIter = <Vec<Entry> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// An intermediate representation of a bibliography entry which is not tied to a specific end format.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// The citation key for the entry
    pub cite: String,

    /// The title or equivalent of the entry
    pub title: String,

    /// The type of entry.
    ///
    /// [`format::Format`] implementations are free to interpret this type as a more specific type
    /// if required by that format.
    pub variant: EntryType,

    /// List of [`Field`]s, which are essentially key-value pairs.
    pub fields: Vec<Field>,
}

/// The type of a bibliography entry.
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
#[non_exhaustive]
pub enum EntryType {
    Article,
    Bill,
    Book,
    Booklet,
    Conference,
    Document,
    InCollection,
    Manual,
    Software,
    Report,
    MasterThesis,
    PhdThesis,
    Paper,
    Webpage,
    Other(String),
}

/// An entry field which is essentially a key value pair.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    /// Name of the entry field.
    pub name: String,
    /// Value of the entry field.
    pub value: String,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn false_on_duplicate_field() {
        let references = Biblio::new(vec![Entry {
            cite: "Edelkamp_2019".to_owned(),
            title: "QuickXsort: A Fast Sorting Scheme in Theory and Practice".to_owned(),
            variant: EntryType::Book,
            fields: vec![Field {
                name: "doi".to_owned(),
                value: "test".to_owned(),
            }],
        }]);

        assert!(references.contains_field("doi", |f| f.value == "test"));
        assert!(!references.contains_field("doi", |f| f.value == "something else"));
    }
}
