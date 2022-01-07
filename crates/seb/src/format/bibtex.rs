use crate::ast::{self, Biblio};

use super::Format;

use biblatex::{Bibliography, ChunksExt};
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
        let entries = biblio.into_iter().map(ast::Entry::from).collect();
        Ok(ast::Biblio::new(entries))
    }

    fn compose(ast: Biblio) -> Self {
        let s = ast
            .into_iter()
            .map(|entry| {
                format!(
                    "@{}{{{},\n    title = {{{}}},\n{}}}\n",
                    compose_type(&entry.variant),
                    entry.cite,
                    entry.title,
                    compose_fields(&entry.fields)
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

const fn compose_type(entry_type: &ast::EntryType) -> &'static str {
    match entry_type {
        ast::EntryType::Article => "article",
        ast::EntryType::Book => "book",
        ast::EntryType::Booklet => "booklet",
        ast::EntryType::Conference => "conference",
        ast::EntryType::InCollection => "incollection",
        ast::EntryType::Manual => "manual",
        ast::EntryType::MasterThesis => "masterthesis",
        ast::EntryType::PhdThesis => "phdthesis",
        ast::EntryType::Report => "techreport",
        _ => "misc",
    }
}

fn compose_fields(fields: &[ast::Field]) -> String {
    fields
        .iter()
        .map(|field| format!("    {} = {{{}}},\n", field.name, field.value))
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

        let variant = match entry_type {
            biblatex::EntryType::Article => ast::EntryType::Article,
            biblatex::EntryType::Book => ast::EntryType::Book,
            biblatex::EntryType::Booklet => ast::EntryType::Booklet,
            biblatex::EntryType::InCollection => ast::EntryType::InCollection,
            biblatex::EntryType::InProceedings => ast::EntryType::Conference,
            biblatex::EntryType::Manual => ast::EntryType::Manual,
            biblatex::EntryType::MastersThesis => ast::EntryType::MasterThesis,
            biblatex::EntryType::PhdThesis => ast::EntryType::PhdThesis,
            biblatex::EntryType::TechReport | biblatex::EntryType::Report => ast::EntryType::Report,
            biblatex::EntryType::Thesis => ast::EntryType::Paper,
            biblatex::EntryType::Online => ast::EntryType::Webpage,
            biblatex::EntryType::Software => ast::EntryType::Software,
            _ => ast::EntryType::Other(entry_type.to_string()),
        };

        let mut fields: Vec<ast::Field> = fields
            .drain()
            .map(|(k, v)| ast::Field {
                name: k,
                value: v.format_verbatim(),
            })
            .collect();

        let index = fields
            .iter()
            .enumerate()
            .find(|(_, f)| f.name == "title" || f.name == "booktitle")
            .map(|(i, _)| i);

        let title = index.map(|i| fields.remove(i).value).unwrap_or_default();

        Self {
            cite,
            title,
            variant,
            fields,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn fields() -> Vec<ast::Field> {
        vec![ast::Field {
            name: "author".to_owned(),
            value: "Me".to_owned(),
        }]
    }

    fn entries() -> Vec<ast::Entry> {
        vec![ast::Entry {
            cite: "entry1".to_owned(),
            title: "Test".to_owned(),
            variant: ast::EntryType::Book,
            fields: fields()[..1].to_vec(),
        }]
    }

    #[test]
    fn parse_then_compose_bibtex() {
        let bibtex_str = include_str!("../../../../tests/data/bibtex1.bib");
        let bibtex = BibTex::new(bibtex_str.to_owned());
        let mut parsed = bibtex.parse().expect("bibtex1.bib is a valid bibtex entry");

        let composed = BibTex::compose(parsed.clone());

        // we don't want to compare bibtex_str with composed raw as they can be different
        let mut parsed_two = composed
            .parse()
            .expect("second parse of composed bibtex1 should be valid");

        // compare the two ASTs as they MUST be the same
        // sort the entries using the test function
        parsed.sort_entries();
        parsed_two.sort_entries();
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
        let expected = "@book{entry1,
    title = {Test},
    author = {Me},
}\n";

        assert_eq!(expected, result.raw());
    }
}
