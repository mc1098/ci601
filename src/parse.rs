//! Trait for parsing generics types to `Bibliography`.

// TODO: expand on mod docs

use biblatex::Bibliography;
use eyre::Result;

/// A trait that performs parsing of the generic type to the `Bibliography`.
pub trait Parser<T> {
    /// Parse a generic type to the `Bibliography`.
    ///
    /// # Errors
    /// When the generic type cannot be parsed into a valid `Bibliography`.
    fn parse(&self, src: T) -> Result<Bibliography>;
}

impl<F, T> Parser<T> for F
where
    F: Fn(T) -> Result<Bibliography>,
{
    fn parse(&self, src: T) -> Result<Bibliography> {
        self(src)
    }
}
