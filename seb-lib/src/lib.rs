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

extern crate self as seb;

mod api;
pub mod ast;
mod error;
pub mod format;

use ast::{Biblio, BiblioResolver};
pub use error::{Error, ErrorKind};

use format::Format;
use log::trace;

type Client = reqwest::blocking::Client;

/// Derive macro for producing all the requirements of an Entry type, most notably implementing the
/// `EntryExt` trait and the resolver.
///
/// The `Entry` macro has some requirements:
/// - The derived item must be a named struct
/// - Struct must also derive `Debug` due to `EntryExt`
/// - The derived struct *MUST* have a field named cite with the type `String`
/// - The derived struct *MUST* have a field named optional with the type [`HashMap<String,
/// QuotedString>`]
///
/// `Entry` macro supports the `kind` attribute which allows for tagging a field as representing
/// the kind type or used on the struct to provide a static string for the kind. If the `kind`
/// attribute is not used then the struct name is normalized to lowercase and is used a static
/// string to represent the kind of entry.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use seb::{ast::QuotedString, Entry};
///
/// #[derive(Debug, Entry)]
/// struct MyExampleEntry {
///     // fields required for the Entry macro
///     cite: String,
///     optional: HashMap<String, QuotedString>,
///     // required fields of the entry itself.
///
///     // Authors of the entry.
///     author: QuotedString,
///     // Title of the entry.
///     title: QuotedString,
/// }
/// ```
/// The example above derives the `EntryExt` and the value of the `EntryExt::kind` would be
/// "myexampleentry" as this is derived from the struct name and then normalized to lowercase.
///
/// ```
/// use std::collections::HashMap;
/// use seb::{ast::QuotedString, Entry};
///
/// #[derive(Debug, Entry)]
/// #[kind = "custom entry"]
/// struct MyExampleEntry {
///     // fields required for the Entry macro
///     cite: String,
///     optional: HashMap<String, QuotedString>,
///     // required fields of the entry itself.
///
///     // Authors of the entry.
///     author: QuotedString,
///     // Title of the entry.
///     title: QuotedString,
/// }
/// ```
/// The example above derives the `EntryExt` and the value of the `EntryExt::kind` would be
/// "custom entry" which
///
/// ```
/// use std::{borrow::Cow, collections::HashMap};
/// use seb::{ast::QuotedString, Entry};
///
/// #[derive(Debug, Entry)]
/// struct MyExampleEntry {
///     // dynamic kind string
///     #[kind]
///     kind: Cow<'static, str>,
///     // fields required for the Entry macro
///     cite: String,
///     optional: HashMap<String, QuotedString>,
///     // required fields of the entry itsefl.
///
///     /// Authors of the entry.
///     author: QuotedString,
///     /// Title of the entry.
///     title: QuotedString,
/// }
/// ```
/// The example above derives the `EntryExt` and the value of the `EntryExt::kind` would be
/// the value of the `kind` field tagged by the kind attribute.
pub use seb_macro::Entry;

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
