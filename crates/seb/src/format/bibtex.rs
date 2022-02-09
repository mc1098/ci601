use crate::{
    ast::{self, Biblio, BiblioResolver, QuotedString},
    Error, ErrorKind,
};

use super::Format;

use biblatex::Bibliography;

/// A type wrapper around [`String`] to represent a `BibTex` format string.
#[derive(Debug)]
pub struct BibTex(String);

impl Format for BibTex {
    fn new(val: String) -> Self {
        Self(val)
    }

    fn parse(self) -> Result<Result<Biblio, BiblioResolver>, Error> {
        let biblio = if self.0.is_empty() {
            Bibliography::new()
        } else {
            Bibliography::parse(&self.0)
                .filter(|b| b.len() != 0)
                .ok_or_else(|| {
                    Error::new(ErrorKind::Deserialize, "Unable to parse string as BibTeX")
                })?
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

fn compose_variant(entry: &ast::Entry) -> &str {
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
        ast::Entry::Other(data) => data.kind(), //"misc",
        ast::Entry::Proceedings(_) => "proceedings",
        ast::Entry::TechReport(_) => "techreport",
        ast::Entry::Unpublished(_) => "unpublished",
    }
}

fn bibtex_esc(s: &str) -> String {
    format!("{{{s}}}")
}

fn compose_fields(fields: &[ast::Field<'_, '_>]) -> String {
    fields
        .iter()
        .map(|field| {
            format!(
                "    {} = {{{}}},\n",
                field.name.replace('_', ""),
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

        let mut resolver = match entry_type.to_bibtex() {
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
            biblatex::EntryType::InBook | biblatex::EntryType::SuppBook => {
                ast::BookSection::resolver_with_cite(cite)
            }
            biblatex::EntryType::Proceedings => ast::Proceedings::resolver_with_cite(cite),
            biblatex::EntryType::Unpublished => ast::Unpublished::resolver_with_cite(cite),
            s => ast::Other::resolver_with_cite(cite, s.to_string()),
        };

        for (name, value) in fields.drain() {
            if name == "booktitle" {
                resolver.book_title(value);
            } else {
                resolver.set_field(&name, value);
            }
        }

        resolver
    }
}

impl From<biblatex::Chunks> for QuotedString {
    fn from(chunks: biblatex::Chunks) -> Self {
        use biblatex::Chunk::{self, Normal, Verbatim};

        // Check last character for an escape and if found then start merging using `merge_escaped`
        fn verbatim_chunk_merge(
            verbatim_str: &mut String,
            chunks: &mut impl Iterator<Item = Chunk>,
        ) {
            if verbatim_str
                .chars()
                .last()
                .map(|c| c == '/')
                .unwrap_or_default()
            {
                merge_escaped(verbatim_str, chunks);
            }
        }

        // biblatex parses the chunks in a way where escaping a verbatim section is possible with
        // '/'. We can't just call ChunksExt::to_biblatex_string because the same issue occurs so
        // no easy out!
        //
        // This method corrects this by merging all chunks up including the following two `Normal`
        // chunks and then includes a final `Verbatim` chunk.
        //
        // The following is an example:
        // "{(HTTP/1.1)}"
        //
        // chunks: [Verbatim("(HTTP/"), Normal("1"), Verbatim("."), Normal("1"), Verbatim(")")];
        //
        // The whole thing should be a single verbatim chunk. The '/' escapes the verbatim chunk
        // and causes the pattern V-N-V-N-V, where V is verbatim and N is normal.
        //
        // We also have to recursively check whether each verbatim part might also be another
        // escape..
        #[inline]
        fn merge_escaped(dest: &mut String, chunks: &mut impl Iterator<Item = Chunk>) {
            let mut normal_count = 0;
            while let Some(chunk) = chunks.next() {
                match chunk {
                    Normal(s) => {
                        normal_count += 1;
                        dest.push_str(&s);
                    }
                    Verbatim(mut s) => {
                        verbatim_chunk_merge(&mut s, chunks);
                        dest.push_str(&s);
                        if normal_count == 2 {
                            return;
                        }
                    }
                }
            }
        }

        let mut parts = vec![];

        let mut chunk_iter = chunks.into_iter();
        while let Some(chunk) = chunk_iter.next() {
            match chunk {
                Verbatim(mut s) => {
                    verbatim_chunk_merge(&mut s, &mut chunk_iter);
                    parts.push((true, s));
                }
                Normal(s) => {
                    parts.push((false, s));
                }
            }
        }

        Self::from_parts(parts)
    }
}

#[cfg(test)]
mod tests {

    use std::{borrow::Cow, collections::HashMap};

    use crate::ast::FieldQuery;

    use super::*;

    fn fields() -> Vec<ast::Field<'static, 'static>> {
        vec![ast::Field {
            name: Cow::Borrowed("author"),
            value: Cow::Owned("Me".into()),
        }]
    }

    fn entries() -> Vec<ast::Entry> {
        vec![ast::Entry::Manual(ast::Manual {
            cite: "entry1".to_owned(),
            title: "Test".into(),
            optional: HashMap::from([("author".to_owned(), "Me".into())]),
        })]
    }

    #[test]
    fn parsing_an_empty_string_returns_an_empty_biblio() {
        let bibtex = BibTex::new(String::new());

        let biblio = bibtex
            .parse()
            .expect("Empty string is a valid BibTeX")
            .expect("Empty string is trivially resolved");

        assert_eq!(Vec::<crate::ast::Entry>::new(), biblio.into_entries());
    }

    #[test]
    fn biblatex_verbatim_chunk_escape_is_corrected() {
        use biblatex::Chunk::{Normal, Verbatim};
        // This test is for the real use case when adding using ietf as the title field is often
        // something like:
        //
        // title = {{Hypertext Transfer Protocol (HTTP/1.1): Authentication}}
        //
        // Notice the double curly braces - the whole title is verbatim and should not be styled
        // differently...however biblatex parses the '/' as an escape and splits up the title into
        // a mix of Verbatim and Normal chunks (per biblatex types).
        //
        // In the From<Chunks> impl we try to correct this by merging the escaped chunks back into
        // the single verbatim for QuotedString.

        // To reduce noise lets reduce the above example to the core problem
        // biblatex will parse the following into the below chunks:
        //
        // "{(HTTP/1.1)}"
        let chunks = vec![
            Verbatim("(HTTP/".to_owned()),
            Normal("1".to_owned()),
            Verbatim(".".to_owned()),
            Normal("1".to_owned()),
            Verbatim(")".to_owned()),
        ];

        let qs = QuotedString::from(chunks);

        assert_eq!("{(HTTP/1.1)}", qs.map_quoted(|s| format!("{{{s}}}")));
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
    fn book_title_in_bibtex_should_be_booktitle() {
        let result = compose_fields(&[ast::Field {
            name: Cow::Borrowed("book_title"),
            value: Cow::Owned("value".into()),
        }]);

        assert_eq!("    booktitle = {value},\n", result);
    }

    #[test]
    fn parse_booktitle_field_as_book_title() {
        let biblio = BibTex::new("@misc{cite, title={title},booktitle={Correct},}".to_owned())
            .parse()
            .expect("Valid BibTeX string")
            .expect("Valid entry fields");

        let entry = biblio.into_entries().remove(0);

        assert_eq!("Correct", &**entry.get_field("book_title").unwrap());
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
