use crate::ast::{self, Biblio, BiblioResolver, QuotedString};

use super::Format;

use biblatex::Bibliography;
use eyre::{eyre, Result};

/// A type wrapper around [`String`] to represent a `BibTex` format string.
#[derive(Debug)]
pub struct BibTex(String);

impl Format for BibTex {
    fn new(val: String) -> Self {
        Self(val)
    }

    fn parse(self) -> Result<std::result::Result<Biblio, BiblioResolver>> {
        let biblio = if self.0.is_empty() {
            Bibliography::new()
        } else {
            Bibliography::parse(&self.0)
                .filter(|b| b.len() != 0)
                .ok_or_else(|| eyre!("Cannot parse the BibTeX"))?
        };
        let entries = biblio.into_iter().map(ast::Resolver::from).collect();
        Ok(Biblio::try_resolve(entries))
    }

    fn compose(ast: Biblio) -> Self {
        let s = ast
            .entries()
            .map(|entry| {
                format!(
                    "@{}{{{},\n{}}}\n",
                    compose_variant(entry),
                    entry.cite(),
                    compose_fields(&entry.fields())
                )
            })
            .collect::<String>();
        Self(s)
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

const fn compose_variant(entry: &ast::Entry) -> &'static str {
    match entry {
        ast::Entry::Article(_) => "article",
        ast::Entry::Book(_) => "book",
        ast::Entry::Booklet(_) => "booklet",
        ast::Entry::BookChapter(_) | ast::Entry::BookPages(_) => "inbook",
        ast::Entry::BookSection(_) => "incollection",
        ast::Entry::InProceedings(_) => "inproceedings",
        ast::Entry::Manual(_) => "manual",
        ast::Entry::MasterThesis(_) => "masterthesis",
        ast::Entry::PhdThesis(_) => "phdthesis",
        ast::Entry::Other(_) => "misc",
        ast::Entry::Proceedings(_) => "proceedings",
        ast::Entry::TechReport(_) => "techreport",
        ast::Entry::Unpublished(_) => "unpublished",
    }
}

fn bibtex_esc(s: &str) -> String {
    format!("{{{s}}}")
}

fn compose_fields(fields: &[ast::Field]) -> String {
    fields
        .iter()
        .map(|field| {
            format!(
                "    {} = {{{}}},\n",
                field.name,
                field.value.map_quoted(bibtex_esc)
            )
        })
        .collect()
}

impl From<biblatex::Entry> for ast::Resolver {
    fn from(entry: biblatex::Entry) -> Self {
        // Deconstruct to avoid cloning
        let biblatex::Entry {
            key: cite,
            entry_type,
            mut fields,
        } = entry;

        let mut resolver = match entry_type {
            biblatex::EntryType::Article => ast::Article::resolver_with_cite(cite),
            biblatex::EntryType::Book => ast::Book::resolver_with_cite(cite),
            biblatex::EntryType::Booklet => ast::Booklet::resolver_with_cite(cite),
            biblatex::EntryType::InCollection => ast::BookSection::resolver_with_cite(cite),
            biblatex::EntryType::InProceedings => ast::InProceedings::resolver_with_cite(cite),
            biblatex::EntryType::Manual => ast::Manual::resolver_with_cite(cite),
            biblatex::EntryType::MastersThesis => ast::MasterThesis::resolver_with_cite(cite),
            biblatex::EntryType::PhdThesis => ast::PhdThesis::resolver_with_cite(cite),
            biblatex::EntryType::TechReport | biblatex::EntryType::Report => {
                ast::TechReport::resolver_with_cite(cite)
            }
            _ => ast::Other::resolver_with_cite(cite),
        };

        for (name, value) in fields.drain() {
            resolver.set_field(&name, value.into());
        }

        resolver
    }
}

impl From<biblatex::Chunks> for QuotedString {
    fn from(chunks: biblatex::Chunks) -> Self {
        let parts = chunks
            .into_iter()
            .map(|c| match c {
                biblatex::Chunk::Verbatim(s) => (true, s),
                biblatex::Chunk::Normal(s) => (false, s),
            })
            .collect();

        Self::from_parts(parts)
    }
}

#[cfg(test)]
mod tests {

    use std::{borrow::Cow, collections::HashMap};

    use super::*;

    fn fields() -> Vec<ast::Field<'static, 'static>> {
        vec![ast::Field {
            name: Cow::Borrowed("author"),
            value: Cow::Owned(QuotedString::new("Me".to_owned())),
        }]
    }

    fn entries() -> Vec<ast::Entry> {
        vec![ast::Entry::Manual(ast::Manual {
            cite: "entry1".to_owned(),
            title: QuotedString::new("Test".to_owned()),
            optional: HashMap::from([("author".to_owned(), QuotedString::new("Me".to_owned()))]),
        })]
    }

    #[test]
    fn parse_then_compose_bibtex() {
        let bibtex_str = include_str!("../../../../tests/data/bibtex1.bib");
        let bibtex = BibTex::new(bibtex_str.to_owned());
        let parsed = bibtex
            .parse()
            .unwrap()
            .expect("bibtex1.bib is a valid bibtex entry");

        let composed = BibTex::compose(parsed.clone());

        // we don't want to compare bibtex_str with composed raw as they can be different
        let parsed_two = composed
            .parse()
            .unwrap()
            .expect("second parse of composed bibtex1 should be valid");

        assert_eq!(parsed, parsed_two);
    }

    #[test]
    fn compose_fields_to_bibtex() {
        let fields = fields();
        let result = compose_fields(&fields);

        assert_eq!("    author = {Me},\n", result);
    }

    #[test]
    fn compose_to_bibtex() {
        let references = Biblio::new(entries().drain(..1).collect());
        let result = BibTex::compose(references);

        // indents and newlines are important in this string so don't format!
        let expected = "@manual{entry1,
    title = {Test},
    author = {Me},
}\n";

        assert_eq!(expected, result.raw());
    }
}
