#[derive(Clone, Debug, PartialEq)]
pub struct Biblio(Vec<Entry>);

impl Biblio {
    pub fn new(entries: Vec<Entry>) -> Self {
        Self(entries)
    }

    pub fn insert(&mut self, entry: Entry) {
        self.0.push(entry);
    }

    pub fn entries(&self) -> &[Entry] {
        &self.0
    }

    /// Sorts the entries and their fields.
    ///
    /// This is a function for use in tests in order to sort items in a [`Vec`] before
    /// a equality check is performed.
    #[cfg(test)]
    pub fn sort_entries(&mut self) {
        for entry in self.0.iter_mut() {
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

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub cite: String,
    pub variant: EntryType,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub value: String,
}
