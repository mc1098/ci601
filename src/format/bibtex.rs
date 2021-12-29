use super::Format;

use biblatex::Bibliography;
use eyre::{eyre, Result};

pub struct BibTex(String);

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

    fn raw(self) -> String {
        self.0
    }

    fn name() -> &'static str {
        "BibTex"
    }

    fn ext() -> &'static str {
        "bib"
    }
}
