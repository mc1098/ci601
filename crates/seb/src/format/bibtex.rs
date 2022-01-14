use crate::ast::{self, Biblio, QuotedString};

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

    fn parse(self) -> Result<Biblio> {
        let biblio =
            Bibliography::parse(&self.0).ok_or_else(|| eyre!("Cannot parse the BibTex"))?;
        dbg!("biblatex::Bibliography = {:?}", &biblio);
        let entries = biblio.into_iter().map(ast::Entry::from).collect();
        Ok(ast::Biblio::new(entries))
    }

    fn compose(ast: Biblio) -> Self {
        let s = ast
            .into_iter()
            .map(|entry| {
                format!(
                    "@{}{{{},\n{}}}\n",
                    compose_type(&entry.entry_data),
                    entry.citation_key,
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

const fn compose_type(entry_data: &ast::EntryData) -> &'static str {
    match entry_data {
        ast::EntryData::Article(_) => "article",
        ast::EntryData::Book(_) => "book",
        ast::EntryData::Booklet(_) => "booklet",
        ast::EntryData::BookChapter(_) | ast::EntryData::BookPages(_) => "inbook",
        ast::EntryData::BookSection(_) => "incollection",
        ast::EntryData::InProceedings(_) => "inproceedings",
        ast::EntryData::Manual(_) => "manual",
        ast::EntryData::MasterThesis(_) => "masterthesis",
        ast::EntryData::PhdThesis(_) => "phdthesis",
        ast::EntryData::Other(_) => "misc",
        ast::EntryData::Proceedings(_) => "proceedings",
        ast::EntryData::TechReport(_) => "techreport",
        ast::EntryData::Unpublished(_) => "unpublished",
    }
}

fn bibtex_esc(s: &str) -> String {
    format!("{{{}}}", s)
}

fn compose_fields(fields: &[ast::Field]) -> String {
    dbg!("fields = {:?}", &fields);
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

impl From<biblatex::Entry> for ast::Entry {
    fn from(entry: biblatex::Entry) -> Self {
        // Deconstruct to avoid cloning
        let biblatex::Entry {
            key: cite,
            entry_type,
            mut fields,
        } = entry;

        let mut builder = match entry_type {
            biblatex::EntryType::Article => ast::Article::builder(),
            biblatex::EntryType::Book => ast::Book::builder(),
            biblatex::EntryType::Booklet => ast::Booklet::builder(),
            biblatex::EntryType::InCollection => ast::BookSection::builder(),
            biblatex::EntryType::InProceedings => ast::InProceedings::builder(),
            biblatex::EntryType::Manual => ast::Manual::builder(),
            biblatex::EntryType::MastersThesis => ast::MasterThesis::builder(),
            biblatex::EntryType::PhdThesis => ast::PhdThesis::builder(),
            biblatex::EntryType::TechReport | biblatex::EntryType::Report => {
                ast::TechReport::builder()
            }
            _ => ast::Other::builder(),
        };

        for (name, value) in fields.drain() {
            builder.set_field(&name, value.into());
        }

        let entry_data = builder.build().expect("Invalid entry data");

        Self {
            citation_key: cite,
            entry_data,
        }
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
        vec![ast::Entry {
            citation_key: "entry1".to_owned(),
            entry_data: ast::EntryData::Manual(ast::Manual {
                title: QuotedString::new("Test".to_owned()),
                optional: HashMap::from([(
                    "author".to_owned(),
                    QuotedString::new("Me".to_owned()),
                )]),
            }),
        }]
    }

    #[test]
    fn parse_then_compose_bibtex() {
        let bibtex_str = include_str!("../../../../tests/data/bibtex1.bib");
        let bibtex = BibTex::new(bibtex_str.to_owned());
        dbg!("{:?}", &bibtex);
        let parsed = bibtex.parse().expect("bibtex1.bib is a valid bibtex entry");

        let composed = BibTex::compose(parsed.clone());

        dbg!("{:?}", &composed);

        // we don't want to compare bibtex_str with composed raw as they can be different
        let parsed_two = composed
            .parse()
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
