//! Contains traits and implementions of the [`Format`], [`Reader`], and [`Writer`] trait.

// TODO: expand on mod doc
use std::marker::PhantomData;

mod bibtex;

use crate::{
    ast::{Biblio, BiblioResolver, EntryExt},
    Error,
};
pub use bibtex::BibTex;

// TODO: Consider defining Format so that it can wrap T types, where T: std::io::Write +
// std::io::Read. This would allow Format to trivially uphold the same type bounds as T and would
// reduce the need for format::Writer + format::Reader.

/// A textual representation that can be parsed into and composed from a [`Biblio`].
///
/// Formats are promises at the type level about what a [`String`] (or similar) represents.
pub trait Format {
    /// Construct a new type using a [`String`] input.
    ///
    /// This function should not panic or fail as creating a [`Format`] is a type promise about
    /// what the [`String`] represents.
    fn new(val: String) -> Self;

    /// Parses this [`Format`] into a [`Biblio`].
    ///
    /// # Errors
    ///
    /// Will return [`Err`] if it's not possible to parse this [`Format`] to [`Biblio`].
    fn parse(self) -> Result<Result<Biblio, BiblioResolver>, Error>;

    /// Composes a [`Biblio`] to this [`Format`].
    ///
    /// This function should not fail as every [`Biblio`] instance must be valid and every
    /// [`Format`] must correctly represent every valid [`Biblio`].
    fn compose(biblio: &Biblio) -> Self;

    /// Composes a [`Entry`] to a [`String`].
    ///
    /// This function should not fail fail as every [`Entry`] instance must be valid and every
    /// [`Format`] must correctly represent every valid [`Entry`].
    fn compose_entry(entry: &dyn EntryExt) -> String;

    /// The current [`Format`] in a raw [`String`].
    ///
    /// Most [`Format`]s are likely to be type wrappers around [`String`] so this is a method to
    /// consume self and get that raw [`String`]. This consumes self and doesn't return a [`str`]
    /// as the [`Format`] might not contain a [`String`] internally so wouldn't be able to return a
    /// reference to one.
    fn raw(self) -> String;

    /// The display name of the format.
    fn name() -> &'static str;

    /// The file extension associated with this format.
    ///
    /// If the format doesn't have a file extension associated then either an empty [`str`] can be
    /// used or the `txt` extension.
    fn ext() -> &'static str;
}

/// A trait for objects which are [`Format`]-oriented sinks.
///
/// Writers are defined by implementing the [`Writer::write`] method which writes a format to this given
/// writer.
///
/// Writers have a default implemention of [`Writer::write_ast`] for [`Biblio`] using the
/// [`Writer::write`] method.
pub trait Writer {
    /// The format associated with the writer.
    type Format: Format;

    /// Write a format into this writer.
    ///
    ///
    /// # Errors
    ///
    /// The call to write should only return an [`Err`] when writing to the writer cannot be
    /// completed.
    fn write(&mut self, format: Self::Format) -> Result<(), Error>;

    /// Write a [`Biblio`] into this writer using [`Format::compose`] from the [`Writer::Format`]
    /// associated type.
    ///
    /// # Errors
    ///
    /// The call to write should only return an [`Err`] when writing to the writer cannot be
    /// completed.
    fn write_ast(&mut self, ast: Biblio) -> Result<(), Error> {
        let format = Self::Format::compose(&ast);
        self.write(format)
    }
}

/// The [`Reader`] trait allows for reading a [`Format`] from a source.
///
/// Readers are defined by implementing the [`Reader::read`] method which reads a format from this given
/// reader.
///
/// Readers have a default implemention of [`Reader::read_ast`] for [`Biblio`] using the [`Reader::read`] method.
pub trait Reader {
    /// The format associated with the reader.
    type Format: Format;

    /// Pull some bytes from this writer in order to produce a [`Reader::Format`] instance.
    ///
    /// # Errors
    /// If this method encounters any form of error making it unable to read the bytes in order to
    /// create the format.
    fn read(&mut self) -> Result<Self::Format, Error>;

    /// Read bytes from this writer using [`Reader::read`] and then parse using [`Format::parse`]
    /// with the associated [`Reader::Format`] type.
    ///
    /// # Errors
    /// This will return [`Err`] if there is an error from [`Reader::read`] or an error when parsing
    /// using [`Format::parse`].
    fn read_ast(&mut self) -> Result<Result<Biblio, BiblioResolver>, Error> {
        let format = self.read()?;
        format.parse()
    }
}

/// A [`String`] wrapper that includes type information of the format the wrapped [`String`]
/// represents.
#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq)]
pub struct FormatString<F: Format> {
    inner: String,
    _format: PhantomData<F>,
}

impl<F: Format> Default for FormatString<F> {
    fn default() -> Self {
        Self {
            inner: String::default(),
            _format: PhantomData,
        }
    }
}

impl<F: Format> FormatString<F> {
    /// Construct a new instance by wrapping an existing [`String`].
    #[must_use]
    pub fn new(val: String) -> Self {
        Self {
            inner: val,
            _format: PhantomData,
        }
    }
}

impl<F: Format> From<FormatString<F>> for String {
    fn from(val: FormatString<F>) -> Self {
        val.inner
    }
}

impl<F: Format> Reader for FormatString<F> {
    type Format = F;

    fn read(&mut self) -> Result<Self::Format, Error> {
        Ok(F::new(self.inner.clone()))
    }
}

impl<F: Format> Writer for FormatString<F> {
    type Format = F;

    fn write(&mut self, format: F) -> Result<(), Error> {
        self.inner.push_str(&format.raw());
        Ok(())
    }
}
