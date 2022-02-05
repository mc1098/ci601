//! Trait for parsing generics types to `Biblio`.

// TODO: expand on mod docs

use crate::{Biblio, Error};

/// A trait that performs parsing of the generic type to the `Biblio`.
pub trait Parser<T> {
    /// Parse a generic type to the `Biblio`.
    ///
    /// # Errors
    /// When the generic type cannot be parsed into a valid `Biblio`.
    fn parse(&self, src: T) -> Result<Biblio, Error>;
}

impl<F, T> Parser<T> for F
where
    F: Fn(T) -> Result<Biblio, Error>,
{
    fn parse(&self, src: T) -> Result<Biblio, Error> {
        self(src)
    }
}
