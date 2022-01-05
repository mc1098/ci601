#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::perf,
    clippy::style,
    clippy::missing_safety_doc,
    clippy::missing_const_for_fn,
    missing_docs
)]
#![allow(clippy::module_name_repetitions)]

//! # bibadd-core
//!
//! bibadd-core is a library which supports searching for bibliography entries from select APIs
//! and adding them to an in memory bibliography. bibadd-core supports transforming the in-memory
//! bibliography to [`format::Format`]s such as [`format::BibTex`].

mod ast;
pub mod format;
pub mod parse;
mod service;

pub use ast::{Biblio, Entry, EntryType, Field};
pub(crate) use service::api;

use eyre::Result;
use log::trace;

/// Seek bibliographic entries by `doi` using the default API.
///
/// Seeking by `doi` should only return a single [`Entry`] but a [`Vec`] is used to provide a
/// consistent API across all `entries_by_*` functions.
///
/// # Errors
///
/// An [`Err`] is returned when no entry is found for the `doi`.
/// An [`Err`] is returned when the response from the API cannot be parsed into a valid [`Entry`].
#[inline]
pub fn entries_by_doi(doi: &str) -> Result<Vec<Entry>> {
    trace!("Seek entries by doi of '{}'", doi);
    api::cross_ref::get_entries_by_doi(doi)
}

/// Seek bibliographic entries by `isbn` using the default API.
///
/// Seeking by `isbn` should only return a single [`Entry`] but a [`Vec`] is used to provide a
/// consistent API across all `entries_by_*` functions.
///
/// # Errors
///
/// An [`Err`] is returned when no entry is found for the `isbn`.
/// An [`Err`] is returned when the response from the API cannot be parsed into a valid [`Entry`].
#[inline]
pub fn entries_by_isbn(isbn: &str) -> Result<Vec<Entry>> {
    trace!("Seek entries by ISBN of '{}'", isbn);
    api::google_books::get_entries_by_isbn(isbn)
}
