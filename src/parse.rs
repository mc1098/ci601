//! Trait for parsing generics types to `Biblio`.

// TODO: expand on mod docs

use crate::Biblio;

use eyre::Result;

/// A trait that performs parsing of the generic type to the `Biblio`.
pub trait Parser<T> {
    /// Parse a generic type to the `Biblio`.
    ///
    /// # Errors
    /// When the generic type cannot be parsed into a valid `Biblio`.
    fn parse(&self, src: T) -> Result<Biblio>;
}

impl<F, T> Parser<T> for F
where
    F: Fn(T) -> Result<Biblio>,
{
    fn parse(&self, src: T) -> Result<Biblio> {
        self(src)
    }
}
