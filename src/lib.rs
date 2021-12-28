pub mod parse;
mod service;

use service::{get_book_by_isbn, get_entry_by_doi};

use biblatex::{Bibliography, ChunksExt, Entry};
use eyre::{eyre, Context, Result};
use log::trace;

#[inline]
fn unique_entry_check<P>(bibliography: Bibliography, predicate: P) -> Result<()>
where
    P: Fn(&Entry) -> Result<()>,
{
    bibliography.iter().try_fold((), |_, e| predicate(e))
}

pub fn add_by_doi(
    doi: &str,
    bib: &mut impl std::io::Write,
    bibliography: Bibliography,
) -> Result<()> {
    trace!("Check if the doi '{}' already exists", doi);
    // check if the current bibliography contains the entry already before doing the http request.
    unique_entry_check(bibliography, |e| {
        e.doi()
            .and_then(|d| (d != doi).then(|| ()))
            .ok_or_else(|| eyre!("A bibtex entry already exists with the doi of '{}'", doi))
    })?;

    let entry = get_entry_by_doi(doi)?;
    bib.write_all(entry.to_bibtex_string().as_bytes())
        .wrap_err_with(|| eyre!("Cannot add entry"))
}

pub fn add_by_isbn(
    isbn: &str,
    bib: &mut impl std::io::Write,
    bibliography: Bibliography,
) -> Result<()> {
    trace!("Check if the ISBN '{}' already exists", isbn);
    unique_entry_check(bibliography, |e| {
        e.isbn()
            .map(|chunk| chunk.format_verbatim().to_lowercase() == isbn)
            .and_then(|b| b.then(|| ()))
            .ok_or_else(|| eyre!("A bibtex entry already exists with the ISBN of '{}'", isbn))
    })?;

    let entry = get_book_by_isbn(isbn)?;
    bib.write_all(entry.to_bibtex_string().as_bytes())
        .wrap_err_with(|| eyre!("Cannot add entry"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BIBTEX_ENTRY_1: &str = include_str!("../tests/data/bibtex1.bib");

    #[test]
    #[should_panic(expected = "Duplicate found")]
    fn err_on_duplicate_entry() {
        let bib = Bibliography::parse(BIBTEX_ENTRY_1).unwrap();
        unique_entry_check(bib, |e| {
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
        add_by_doi("10.1007/s00453-019-00634-0", &mut Vec::new(), bib).unwrap();
    }
}
