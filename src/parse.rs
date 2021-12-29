use biblatex::Bibliography;
use eyre::Result;

use crate::format::Format;

pub trait Parser<T> {
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

pub trait Compose<F: Format> {
    fn compose(&self, ast: Bibliography) -> Result<F>;
}
