#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::perf,
    clippy::style,
    clippy::missing_safety_doc,
    clippy::missing_const_for_fn,
    missing_docs
)]

//! # bibadd-core
//!
//! bibadd-core is a library which supports searching for bibliography entries from select APIs
//! and adding them to an in memory bibliography. bibadd-core supports transforming the in-memory
//! bibliography to [`format::Format`]s such as [`format::BibTex`].

mod ast;
pub mod format;
pub mod parse;
mod service;

use ast::{Biblio, Entry};
use format::Writer;
use service::{get_book_by_isbn, get_entry_by_doi};

use eyre::{eyre, Result};
use log::trace;

#[inline]
fn unique_entry_check<P>(references: &Biblio, predicate: P) -> Result<()>
where
    P: Fn(&Entry) -> Result<()>,
{
    references
        .entries()
        .iter()
        .try_fold((), |_, e| predicate(e))
}

/// Seek a bibliography entry by doi and then write it to the writer if it doesn't already exists
/// in the current bibliography.
///
/// # Errors
///
/// All errors are [`eyre::Report`]s so are not designed to be caught but to be propagated up.
///
/// Duplicate entry:
/// When an entry with the given doi already exists.
///
/// Doi not found:
/// When the doi is not valid or a resource cannot be found the given doi.
///
/// Doi resource found but cannot be parsed correctly:
/// When a resource was found for the doi but the returned information could not parsed correctly
/// to create a valid [`Biblio`].
///
/// Writer error:
/// An error in the writer when trying to write the new entry to the writer.
///
pub fn add_by_doi<F: Writer>(doi: &str, writer: &mut F, mut references: Biblio) -> Result<()> {
    unique_entry_check(&references, |e| {
        e.fields
            .iter()
            .filter(|f| f.name == "doi" && f.value != doi)
            .map(|_| ())
            .next()
            .ok_or_else(|| eyre!("An entry already exists with the doi of '{}'", doi))
    })?;

    let entry = get_entry_by_doi(doi)?;
    references.insert(entry);
    writer.write_ast(references)
}

/// Seek a bibliography entry by isbn and then write it to the writer if it doesn't already exists
/// in the current bibliography.
///
/// # Errors
///
/// All errors are [`eyre::Report`]s so are not designed to be caught but to be propagated up.
///
/// Duplicate entry:
/// When an entry with the given isbn already exists.
///
/// ISBN not found:
/// When the ISBN is not valid or a resource cannot be found the given doi.
///
/// ISBN resource found but cannot be parsed correctly:
/// When a resource was found for the ISBN but the returned information could not parsed correctly
/// to create a valid [`Biblio`].
///
/// Writer error:
/// An error in the writer when trying to write the new entry to the writer.
///
pub fn add_by_isbn<F: Writer>(isbn: &str, writer: &mut F, mut references: Biblio) -> Result<()> {
    trace!("Check if the ISBN '{}' already exists", isbn);
    unique_entry_check(&references, |e| {
        e.fields
            .iter()
            .filter(|f| f.name == "isbn" && f.value == isbn)
            .map(|_| ())
            .next()
            .ok_or_else(|| eyre!("An entry already exists with the ISBN of '{}'", isbn))
    })?;

    let entry = get_book_by_isbn(isbn)?;
    references.insert(entry);
    writer.write_ast(references)
}

#[cfg(test)]
mod tests {
    use crate::format::{BibTex, Format, FormatString};

    use super::*;

    const BIBTEX_ENTRY_1: &str = include_str!("../tests/data/bibtex1.bib");

    #[test]
    #[should_panic(expected = "Duplicate found")]
    fn err_on_duplicate_entry() {
        let references = Biblio::new(vec![Entry {
            cite: "Edelkamp_2019".to_owned(),
            variant: ast::EntryType::Book,
            fields: vec![],
        }]);
        unique_entry_check(&references, |e| {
            if e.cite == "Edelkamp_2019" {
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
        let references = BibTex::new(BIBTEX_ENTRY_1.to_owned()).parse().unwrap();
        add_by_doi(
            "10.1007/s00453-019-00634-0",
            &mut FormatString::<BibTex>::default(),
            references,
        )
        .unwrap();
    }
}
