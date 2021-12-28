use std::ops::Deref;

use biblatex::Bibliography;
use eyre::{eyre, Result};

pub trait Format {
    fn new(val: String) -> Self;
    fn parse(self) -> Result<Bibliography>;
    fn compose(ast: Bibliography) -> Self;
    fn name() -> &'static str;
    fn ext() -> &'static str;
}

pub struct BibTex(String);

impl Deref for BibTex {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Format for BibTex {
    fn new(val: String) -> Self {
        Self(val)
    }

    fn parse(self) -> Result<Bibliography> {
        Bibliography::parse(&self.0).ok_or_else(|| eyre!("Cannot parse the BibTex"))
    }

    fn compose(ast: Bibliography) -> Self {
        Self(ast.to_bibtex_string())
    }

    fn name() -> &'static str {
        "BibTex"
    }

    fn ext() -> &'static str {
        ".bib"
    }
}

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
