#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::perf,
    clippy::style,
    clippy::missing_safety_doc,
    clippy::missing_const_for_fn
)]
#![warn(missing_docs, rust_2018_idioms)]
#![allow(clippy::module_name_repetitions)]
#![doc = include_str!("../README.md")]

mod api;
pub mod ast;
mod error;
pub mod format;

use ast::{Biblio, BiblioResolver};
pub use error::{Error, ErrorKind};

use format::Format;
use log::trace;

type Client = reqwest::blocking::Client;

/// Search bibliographic entries by `doi` using the default API.
///
/// Searching by `doi` should only return a single [Entry][E] but a [`Vec`] is used to provide a
/// consistent API across all `entries_by_*` functions.
///
///
/// # Errors
///
/// An `Err` is returned when no entry is found for the `doi`.
/// An `Err` is returned when the response from the API cannot be parsed into a valid [Entry][E].
///
/// [E]: ast::Entry
#[inline]
pub fn entries_by_doi(doi: &str) -> Result<Result<Biblio, BiblioResolver>, Error> {
    trace!("Search entries by doi of '{doi}'");
    api::cross_ref::get_entries_by_doi::<Client>(doi)
}

/// Search bibliographic entries by `isbn` using the default API.
///
/// Searching by `isbn` should only return a single [Entry][E] but a [`Vec`] is used to provide a
/// consistent API across all `entries_by_*` functions.
///
/// # Errors
///
/// An `Err` is returned when no entry is found for the `isbn`.
/// An `Err` is returned when the response from the API cannot be parsed into a valid [Entry][E].
///
/// [E]: ast::Entry
#[inline]
pub fn entries_by_isbn(isbn: &str) -> Result<Result<Biblio, BiblioResolver>, Error> {
    trace!("Search entries by ISBN of '{isbn}'");
    api::google_books::get_entries_by_isbn::<Client>(isbn)
}

/// Search bibliographic entries by `IETF RFC number`.
///
/// Searching by `IETF RFC number` should only return a single [Entry][E] but a [`Vec`] is used to
/// provide a consistent API across all `entries_by_*` functions.
///
/// # Errors
///
/// An `Err` is returned when no entry is found for the RFC number.
/// An `Err` is returned when an error occurs trying to retrive the textual data from the url.
/// An `Err` is returned when the response from the API cannot be parsed into a valid [Entry][E].
///
/// [E]: ast::Entry
#[inline]
pub fn entries_by_rfc(number: usize) -> Result<Result<Biblio, BiblioResolver>, Error> {
    trace!("Search entries by IETF RFC number '{number}'");
    api::ietf::get_entry_by_rfc::<Client>(number)
}

/// Search bibliographic entries at a given `url` when the expected text format matches the `F:
/// Format` used when calling this function.
///
/// # Errors
///
/// An `Err` is returned when no entry is found at the `url`.
/// An `Err` is returned when an error occurs trying to retrive the textual data from the url.
/// An `Err` is returned when the response from the API cannot be parsed into a valid [Entry][E].
///
/// [E]: ast::Entry
#[inline]
pub fn entries_by_url<F: Format>(url: &str) -> Result<Result<Biblio, BiblioResolver>, Error> {
    trace!("Search entries at url of '{url}'");
    api::format_api::get_entry_by_url::<Client, F>(url)
}

/// # Errors
pub fn entry_stubs_by_title(title: &str) -> Result<Vec<(String, String)>, Error> {
    trace!("Search entries that have a title of '{title}'");
    api::cross_ref::get_entry_stubs_by_title::<Client>(title)
}
