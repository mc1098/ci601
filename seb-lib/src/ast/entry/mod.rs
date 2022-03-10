use std::{borrow::Cow, collections::HashMap, fmt::Debug};

use super::{Field, QuotedString};

mod resolver;

pub use resolver::*;

/// Trait for representing both resolved and unresolved entry types.
pub trait EntryExt: Debug {
    /// Returns the type of the Entry.
    ///
    /// This can be used to help identify the entry type, especially when dealing with a trait
    /// object of `EntryExt`.
    fn kind(&self) -> &str;

    /// Searches for a field value that matches the `name` given.
    ///
    /// [`Self::get_field`] returns `Some(&QuotedString)` when a matching field is found
    /// and the return is the value of that matching field, returns `None` when no field
    /// matches the `name`.
    fn get_field(&self, name: &str) -> Option<&QuotedString>;

    /// Returns the citation key of this entry.
    fn cite(&self) -> Cow<'_, str>;

    /// Sets the citation key of this entry to a new value and returns the existing.
    fn set_cite(&mut self, cite: String) -> String;

    /// Returns the `title` field value of this entry.
    ///
    /// Entry titles provide a textual representation of the bibliographic entry itself and for
    /// this crate should not be empty for resolved entry types.
    fn title(&self) -> &QuotedString {
        // default impl simply gets and tries to unwrap.
        self.get_field("title").expect(
            "Title is a requirement for all Entry types for seb but was not included on this entry",
        )
    }

    /// Returns the [`Field`]s of the entry.
    ///
    /// The fields returned include the required and optional fields in no particular
    /// order.
    fn fields(&self) -> Vec<Field<'_>>;

    /// Returns true if two instances of this trait are equal.
    fn eq(&self, other: &dyn EntryExt) -> bool {
        for field in self.fields() {
            if other.get_field(&field.name).is_none() {
                return false;
            }
        }
        self.cite() == other.cite()
    }
}

macro_rules! entry_impl {
    ($(
        $mod:ident:
            $(#[$target_comment:meta])*
            $target:ident($(
                $(#[$req_comment:meta])*
                $req:ident
            ),+)
    ),* $(,)?) => {

        /// Modular that includes preset global variables that represent different entry types.
        pub mod kind {
            $(
                $(#[$target_comment])*
                #[allow(non_upper_case_globals)]
                pub const $target: &str = stringify!($mod);
            )*
        }

        $(
            pub use $mod::*;
            mod $mod {
                use super::*;

                $(#[$target_comment])*
                #[derive(Debug, PartialEq)]
                #[cfg_attr(test, derive(Clone))]
                pub struct $target {
                    /// Citation key of the entry
                    pub cite: String,

                    /// The name of the kind.
                    pub kind: Cow<'static, str>,

                    $(
                        $(#[$req_comment])*
                        pub $req: QuotedString,
                    )+
                    /// Optional fields that a not essential for creating a valid entry of this
                    /// type.
                    pub optional: HashMap<String, QuotedString>,
                }

                impl $target {

                    /// Creates a new [`Resolver`] for this type to ensure that the required fields
                    /// are set before the entry type can be built.
                    ///
                    /// Does not set the cite value of the resolver so will be generated based on
                    /// the field values.
                    #[must_use]
                    pub fn resolver() -> Resolver {
                        Resolver {
                            kind: Cow::Borrowed(stringify!($mod)),
                            cite: None,
                            req: [$(stringify!($req),)+].to_vec(),
                            fields: HashMap::default(),
                            entry_resolve: resolve,
                        }
                    }

                    /// Creates a new [`Resolver`] for this type to ensure that the required fields
                    /// are set before the entry type can be built.
                    #[must_use]
                    pub fn resolver_with_cite<S: Into<String>>(cite: S) -> Resolver {
                        Resolver {
                            kind: Cow::Borrowed(stringify!($mod)),
                            cite: Some(cite.into()),
                            req: [$(stringify!($req),)+].to_vec(),
                            fields: HashMap::default(),
                            entry_resolve: resolve,
                        }
                    }
                }


                fn resolve(mut resolver: Resolver) -> Box<dyn EntryExt> {
                    let cite = resolver.cite().to_string();

                    let data = $target {
                        kind: resolver.kind,
                        cite,
                        $($req: resolver.fields.remove(stringify!($req)).unwrap(),)+
                        optional: resolver.fields,
                    };

                    Box::new(data)

                }

                impl EntryExt for $target {
                    fn kind(&self) -> &str {
                        &self.kind
                    }

                    fn get_field(&self, name: &str) -> Option<&QuotedString> {
                        let normal_name = name.to_lowercase();
                        match normal_name.as_str() {
                            $(stringify!($req) => Some(&self.$req),)+
                            s => self.optional.get(s),
                        }
                    }

                    fn cite(&self) -> Cow<'_, str> {
                        Cow::Borrowed(&self.cite)
                    }

                    fn set_cite(&mut self, cite: String) -> String {
                        std::mem::replace(&mut self.cite, cite)
                    }

                    fn fields(&self) -> Vec<Field<'_>> {
                        let mut fields: Vec<_> = [$((stringify!($req), &self.$req),)+]
                            .into_iter()
                            .map(Field::from)
                            .collect();
                        fields.extend(self.optional.iter().map(Field::from));
                        fields
                    }
                }

                #[test]
                fn resolver_override_cite() {
                    use std::collections::VecDeque;

                    let mut resolver = $target::resolver_with_cite("old");
                    let mut required: VecDeque<_> = [$(stringify!($req),)+].into_iter().collect();

                    let iter = std::iter::from_fn(move || required.pop_front());
                    let iter = iter.zip(('a'..).into_iter());

                    for (field, c) in iter {
                        resolver.set_field(field, c.to_string());
                    }
                    let res = resolver.resolve();
                    let mut entry = res.expect("All required fields added so should have built correctly");

                    assert_eq!("old", entry.cite());
                    entry.set_cite("new".to_owned());
                    assert_eq!("new", entry.cite());
                }

                #[test]
                fn resolver_only_returns_ok_when_all_required_fields_set() {
                    use std::collections::VecDeque;

                    let mut resolver = $target::resolver();
                    let mut required: VecDeque<_> = [$(stringify!($req),)+].into_iter().collect();

                    let iter = std::iter::from_fn(move || required.pop_front());
                    let iter = iter.zip(('a'..).into_iter());

                    for (field, c) in iter {
                        resolver = resolver
                            .resolve()
                            .expect_err("Resolver should not resolve correctly without required fields");

                        resolver.set_field(field, c.to_string());
                    }
                    resolver.set_field("test", "value");
                    let res = resolver.resolve();
                    let entry = res.expect("All required fields added so should have built correctly");

                    let mut alpha = ('a'..).into_iter().map(|c| c.to_string());

                    $(
                        let expected = alpha.next().unwrap();
                        let field = entry.get_field(stringify!($req)).unwrap();
                        assert_eq!(expected, field.as_ref());
                    )+
                }

            }
        )*
    }
}

/// Any other resource not supported by other entry variants.
#[derive(Clone, Debug, PartialEq)]
pub struct Other {
    cite: String,
    kind: Cow<'static, str>,
    title: QuotedString,
    optional: HashMap<String, QuotedString>,
}

impl Other {
    /// The type of this custom entry.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Creates a new [`Resolver`] for this type to ensure that the required fields
    /// are set before the entry type can be built.
    #[must_use]
    pub fn resolver(kind: String) -> Resolver {
        Resolver {
            kind: Cow::Owned(kind),
            cite: None,
            req: vec!["title"],
            fields: HashMap::new(),
            entry_resolve: Self::resolve,
        }
    }

    /// Creates a new [`Resolver`] for this type to ensure that the required fields
    /// are set before the entry type can be built.
    #[must_use]
    pub fn resolver_with_cite<S: Into<String>>(kind: String, cite: S) -> Resolver {
        Resolver {
            kind: Cow::Owned(kind),
            cite: Some(cite.into()),
            req: vec!["title"],
            fields: HashMap::new(),
            entry_resolve: Self::resolve,
        }
    }

    fn resolve(mut resolver: Resolver) -> Box<dyn EntryExt> {
        Box::new(Other {
            cite: resolver.cite().to_string(),
            kind: resolver.kind,
            title: resolver.fields.remove("title").unwrap(),
            optional: resolver.fields,
        })
    }
    /// Returns the [`Field`]s of the entry.
    ///
    /// The fields returned include the required and optional fields in no particular
    /// order.
    #[must_use]
    pub fn fields(&self) -> Vec<Field<'_>> {
        let field = Field::from(("title", &self.title));
        let mut fields = vec![field];
        fields.extend(self.optional.iter().map(Field::from));
        fields
    }
}

impl EntryExt for Other {
    fn kind(&self) -> &str {
        &self.kind
    }

    fn get_field(&self, name: &str) -> Option<&QuotedString> {
        let normal_name = name.to_lowercase();
        match normal_name.as_str() {
            "title" => Some(&self.title),
            s => self.optional.get(s),
        }
    }

    fn fields(&self) -> Vec<Field<'_>> {
        let field = Field::from(("title", &self.title));
        let mut fields = vec![field];
        fields.extend(self.optional.iter().map(Field::from));
        fields
    }

    fn cite(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.cite)
    }

    fn set_cite(&mut self, cite: String) -> String {
        std::mem::replace(&mut self.cite, cite)
    }
}

entry_impl! {
    article:
        /// The article entry type represents an article
        Article(
            /// Authors of the article.
            author,
            /// Title of the article.
            title,
            /// The journal that contains this article.
            journal,
            /// The year of this article.
            year
        ),
    book:
        /// The book entry type
        Book(
            /// Authors of the book.
            author,
            /// Title of the book.
            title,
            /// The publisher of the book.
            publisher,
            /// The year the book was published.
            year
        ),
    booklet:
        /// The booklet entry type
        Booklet(
            /// Title of the booklet.
            title
        ),
    //inbook
    book_chapter:
        /// A chapter of a book
        BookChapter(
            /// Authors of the book.
            author,
            /// Title of the book.
            title,
            /// Name of the chapter.
            chapter,
            /// Publisher of the book.
            publisher,
            /// Year the book was published.
            year
        ),
    book_pages:
        /// A page range of a book
        BookPages(
            /// Authors of the book.
            author,
            /// Title of the book.
            title,
            /// Page range of the book.
            ///
            /// The range should be in the format of "10-20".
            pages,
            /// Publisher of the book.
            publisher,
            /// Year the book was published.
            year
        ),
    book_section:
        /// A section of a book with a title.
        BookSection(
            /// Authors of the book.
            author,
            /// Title of the section.
            title,
            /// Title of the book.
            book_title,
            /// Publisher of the book.
            publisher,
            /// Year the book was published.
            year
        ),
    in_proceedings:
        /// Published paper in a conference proceedings.
        InProceedings(
            /// Authors of the book.
            author,
            /// Title of the conference.
            title,
            /// Title of the paper.
            book_title,
            /// Year the paper was published.
            year
        ),
    manual:
        /// Manual for technical information for machine software.
        Manual(
            /// Title of the manual.
            title
        ),
    master_thesis:
        /// A thesis for a Master's level degree.
        MasterThesis(
            /// Authors of the thesis.
            author,
            /// Title of the thesis.
            title,
            /// School of the author.
            school,
            /// Year the paper was published.
            year
        ),
    phd_thesis:
        /// A thesis for a PhD level degree.
        PhdThesis(
            /// Authors of the thesis.
            author,
            /// Title of the thesis.
            title,
            /// School of the author.
            school,
            /// Year the paper was published.
            year
        ),
    proceedings:
        /// A conference proceeding.
        Proceedings(
            /// Title of the conference.
            title,
            /// Year of the conference.
            year
        ),
    tech_report:
        /// A technical report.
        TechReport(
            /// Authors of the report.
            author,
            /// Title of the report.
            title,
            /// Institution that published the report.
            institution,
            /// Year of the report.
            year
        ),
    unpublished:
        /// A document that has not been officially published.
        Unpublished(
            /// Authors of the document.
            author,
            /// Title of the document.
            title
        ),
}
