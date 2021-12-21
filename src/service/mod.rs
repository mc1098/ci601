mod doi;

pub use doi::DoiService;

use biblatex::Entry;

pub trait BibTexService {
    fn get_bibtex(&self) -> Result<Entry, eyre::Report>;
}
