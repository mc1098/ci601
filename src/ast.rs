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
