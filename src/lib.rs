pub mod format;
pub mod parse;
mod service;

use format::FormatWriter;
use service::{get_book_by_isbn, get_entry_by_doi};

use biblatex::{Bibliography, ChunksExt, Entry};
use eyre::{eyre, Result};
use log::trace;

#[inline]
fn unique_entry_check<P>(bibliography: &Bibliography, predicate: P) -> Result<()>
where
    P: Fn(&Entry) -> Result<()>,
{
    bibliography.iter().try_fold((), |_, e| predicate(e))
}

pub fn add_by_doi<F: FormatWriter>(
    doi: &str,
    file: &mut F,
    mut bibliography: Bibliography,
) -> Result<()> {
    unique_entry_check(&bibliography, |e| {
        e.doi()
            .and_then(|d| (d != doi).then(|| ()))
            .ok_or_else(|| eyre!("An entry already exists with the doi of '{}'", doi))
    })?;

    let entry = get_entry_by_doi(doi)?;
    bibliography.insert(entry);
    file.write_ast(bibliography)
}

pub fn add_by_isbn<F: FormatWriter>(
    isbn: &str,
    file: &mut F,
    mut bibliography: Bibliography,
) -> Result<()> {
    trace!("Check if the ISBN '{}' already exists", isbn);
    unique_entry_check(&bibliography, |e| {
        e.isbn()
            .map(|chunk| chunk.format_verbatim().to_lowercase() == isbn)
            .and_then(|b| b.then(|| ()))
            .ok_or_else(|| eyre!("An entry already exists with the ISBN of '{}'", isbn))
    })?;

    let entry = get_book_by_isbn(isbn)?;
    bibliography.insert(entry);
    file.write_ast(bibliography)
}

#[cfg(test)]
mod tests {
    use crate::format::{BibTex, FormatString};

    use super::*;

    const BIBTEX_ENTRY_1: &str = include_str!("../tests/data/bibtex1.bib");

    #[test]
    #[should_panic(expected = "Duplicate found")]
    fn err_on_duplicate_entry() {
        let bib = Bibliography::parse(BIBTEX_ENTRY_1).unwrap();
        unique_entry_check(&bib, |e| {
            if e.key == "Edelkamp_2019" {
                Err(eyre!("Duplicate found"))
            } else {
                Ok(())
            }
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "already exists with the doi")]
    fn duplicate_add_on_doi_errors() {
        let bib = Bibliography::parse(BIBTEX_ENTRY_1).unwrap();
        add_by_doi(
            "10.1007/s00453-019-00634-0",
            &mut FormatString::<BibTex>::default(),
            bib,
        )
        .unwrap();
    }
}
