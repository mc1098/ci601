use std::marker::PhantomData;

mod bibtex;

use crate::ast::Biblio;
pub use bibtex::BibTex;

use eyre::Result;

pub trait Format {
    fn new(val: String) -> Self;
    fn parse(self) -> Result<Biblio>;
    fn compose(ast: Biblio) -> Self;
    fn raw(self) -> String;
    fn name() -> &'static str;
    fn ext() -> &'static str;
}

pub trait FormatWriter {
    type Format: Format;

    fn write(&mut self, format: Self::Format) -> Result<()>;

    fn write_ast(&mut self, ast: Biblio) -> Result<()> {
        let format = Self::Format::compose(ast);
        self.write(format)
    }
}

pub trait FormatReader {
    type Format: Format;

    fn read(&mut self) -> Result<Self::Format>;

    fn read_ast(&mut self) -> Result<Biblio> {
        let format = self.read()?;
        format.parse()
    }
}

#[derive(PartialEq)]
pub struct FormatString<F: Format> {
    inner: String,
    _format: PhantomData<F>,
}

impl<F: Format> Default for FormatString<F> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _format: PhantomData,
        }
    }
}

impl<F: Format> FormatString<F> {
    pub fn new(val: String) -> Self {
        Self {
            inner: val,
            _format: PhantomData,
        }
    }

    pub fn into_string(self) -> String {
        self.inner
    }
}

impl<F: Format> FormatReader for FormatString<F> {
    type Format = F;

    fn read(&mut self) -> Result<Self::Format> {
        Ok(F::new(self.inner.clone()))
    }
}

impl<F: Format> FormatWriter for FormatString<F> {
    type Format = F;

    fn write(&mut self, format: F) -> Result<()> {
        self.inner.push_str(&format.raw());
        Ok(())
    }
}
