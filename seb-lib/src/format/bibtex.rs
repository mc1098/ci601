use std::collections::HashMap;

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

    fn compose(biblio: &Biblio) -> Self {
        let mut map = HashMap::new();

        biblio
            .entries()
            .map(|entry| (compose_variant(entry), Self::compose_entry(entry)))
            .for_each(|(kind, entry)| {
                map.entry(kind)
                    .and_modify(|s: &mut String| s.push_str(&entry))
                    .or_insert(format!("% {kind}\n{entry}\n"));
            });

        let mut pairs = map.into_iter().collect::<Vec<_>>();
        pairs.sort_by_key(|(k, _)| *k);

        let bib = pairs.into_iter().map(|(_, groups)| groups).collect();

        Self(bib)
    }

    fn compose_entry(entry: &ast::Entry) -> String {
        format!(
            "@{}{{{},\n{}}}\n",
            compose_variant(entry),
            entry.cite(),
            compose_fields(&entry.fields())
        )
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
        ast::Entry::Other(data) => data.kind(),
        ast::Entry::Proceedings(_) => "proceedings",
        ast::Entry::TechReport(_) => "techreport",
        ast::Entry::Unpublished(_) => "unpublished",
    }
}

fn bibtex_esc(s: &str) -> String {
    format!("{{{s}}}")
}

fn compose_fields(fields: &[ast::Field<'_>]) -> String {
    fields
        .iter()
        .map(|field| {
            let field = compose_field(field);
            format!("    {field},\n")
        })
        .collect()
}

fn compose_field(field: &ast::Field<'_>) -> String {
    match field.name.replace('_', "").as_str() {
        "month" => to_short_month(&field.value),
        name => format!("{name} = {{{}}}", field.value.map_quoted(bibtex_esc)),
    }
}

fn to_short_month(month: &QuotedString) -> String {
    let value = match month.parse() {
        Ok(1) => "jan",
        Ok(2) => "feb",
        Ok(3) => "mar",
        Ok(4) => "apr",
        Ok(5) => "may",
        Ok(6) => "jun",
        Ok(7) => "jul",
        Ok(8) => "aug",
        Ok(9) => "sep",
        Ok(10) => "oct",
        Ok(11) => "nov",
        Ok(12) => "dec",
        _ => month.get(0..3).expect("invalid month value"),
    }
    .to_lowercase();

    format!("month = {value}")
}

impl From<biblatex::EntryType> for ast::EntryKind<'static> {
    fn from(entry_type: biblatex::EntryType) -> Self {
        use ast::EntryKind;
        use biblatex::EntryType;

        match entry_type.to_bibtex() {
            EntryType::Article => EntryKind::Article,
            EntryType::Book => EntryKind::Book,
            EntryType::Booklet => EntryKind::Booklet,
            EntryType::InCollection | EntryType::InBook | EntryType::SuppBook => {
                EntryKind::BookSection
            }
            EntryType::InProceedings => EntryKind::InProceedings,
            EntryType::Manual => EntryKind::Manual,
            EntryType::MastersThesis => EntryKind::MasterThesis,
            EntryType::PhdThesis => EntryKind::PhdThesis,
            EntryType::TechReport | EntryType::Report => EntryKind::TechReport,
            EntryType::Proceedings => EntryKind::Proceedings,
            EntryType::Unpublished => EntryKind::Unpublished,
            s => EntryKind::Other(std::borrow::Cow::Owned(s.to_string())),
        }
    }
}

impl From<biblatex::Entry> for ast::Resolver {
    fn from(entry: biblatex::Entry) -> Self {
        // Deconstruct to avoid cloning
        let biblatex::Entry {
            key: cite,
            entry_type,
            mut fields,
        } = entry;

        let kind = entry_type.into();
        let mut resolver = ast::Entry::resolver_with_cite(kind, cite);

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

    fn fields() -> Vec<ast::Field<'static>> {
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

    /// TODO: normalize field values so that this test is not so fragile
    // #[test]
    // fn parse_then_compose_bibtex() {
    //     let bibtex_str = include_str!("../../tests/data/bibtex1.bib");
    //     let bibtex = BibTex::new(bibtex_str.to_owned());
    //     let parsed = bibtex
    //         .parse()
    //         .unwrap()
    //         .expect("bibtex1.bib is a valid bibtex entry");

    //     let composed = BibTex::compose(&parsed);

    //     // we don't want to compare bibtex_str with composed raw as they can be different
    //     let parsed_two = composed
    //         .parse()
    //         .unwrap()
    //         .expect("second parse of composed bibtex1 should be valid");

    //     assert_eq!(parsed, parsed_two);
    // }

    macro_rules! field {
        ($name:literal: $value:literal) => {
            ast::Field {
                name: Cow::Borrowed($name),
                value: Cow::Owned($value.into()),
            }
        };
        ($name:literal: $value:ident) => {
            ast::Field {
                name: Cow::Borrowed($name),
                value: Cow::Owned($value.into()),
            }
        };
    }

    #[test]
    fn compose_month_name_field_in_short_name_without_braces() {
        let long_month_names = [
            ("jan", "January"),
            ("feb", "Febuary"),
            ("mar", "march"), // shouldn't matter what the case is
            ("oct", "oCtober"),
            ("dec", "December"),
        ];

        check_each_field_with_expected(long_month_names);
    }

    #[test]
    fn compose_month_num_field_in_short_name_without_braces() {
        let month_nums = [
            ("jan", "01"),
            ("apr", "4"),
            ("may", "05"),
            ("aug", "8"),
            ("nov", "11"),
        ];

        check_each_field_with_expected(month_nums);
    }

    #[test]
    fn normalize_date_fields_to_year_month_day_fields() {
        let raw = "@misc{cite, title={test}, date={2020-04-03},}";
        let [year, month, day] = parse_and_get_entry_date_parts(raw);

        assert_eq!("2020", &*year.unwrap());
        assert_eq!("4", &*month.unwrap());
        assert_eq!("3", &*day.unwrap());

        let raw = "@misc{cite, title={test}, date={2022-01},}";
        let [year, month, day] = parse_and_get_entry_date_parts(raw);

        assert_eq!("2022", &*year.unwrap());
        assert_eq!("1", &*month.unwrap());
        assert_eq!(None, day);
    }

    fn parse_and_get_entry_date_parts(raw: &str) -> [Option<QuotedString>; 3] {
        let bib = BibTex::new(raw.to_owned());

        let biblio = bib
            .parse()
            .expect("valid BibTeX string")
            .expect("valid required fields");

        let entry = biblio
            .entries()
            .next()
            .expect("Should contain a single entry");

        let year = entry.get_field("year");
        let month = entry.get_field("month");
        let day = entry.get_field("day");

        [year.cloned(), month.cloned(), day.cloned()]
    }

    fn check_each_field_with_expected<const N: usize>(slice: [(&'static str, &'static str); N]) {
        for (expected_month, month_value) in slice {
            let field = field! { "month": month_value };
            let actual = compose_field(&field);

            assert_eq!(format!("month = {expected_month}"), actual);
        }
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
        let result = BibTex::compose(&references);

        // indents and newlines are important in this string so don't format!
        let expected = "% manual\n@manual{entry1,
    title = {Test},
    author = {Me},
}\n\n";

        assert_eq!(expected, result.raw());
    }
}
