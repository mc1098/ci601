use std::{borrow::Cow, collections::HashMap};

use super::{Field, QuotedString};

mod resolver;

pub use resolver::*;

/// Trait for querying data structures with fields.
pub trait FieldQuery {
    /// Searches for a field value that matches the `name` given.
    ///
    /// [`Self::find_field`] returns `Some(&QuotedString)` when a matching field is found
    /// and the return is the value of that matching field, returns `None` when no field
    /// matches the `name`.
    fn get_field(&self, name: &str) -> Option<&QuotedString>;
}

const fn tuple_to_field<'name, 'value>(
    (name, value): (&'name str, &'value QuotedString),
) -> Field<'name, 'value> {
    Field {
        name: Cow::Borrowed(name),
        value: Cow::Borrowed(value),
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
        /// An intermediate representation of a bibliography entry which is not tied to a specific end
        /// format.
        #[derive(Debug, PartialEq)]
        #[cfg_attr(test, derive(Clone))]
        pub enum Entry {
            $(
                $(#[$target_comment])*
                $target($target),
            )*
        }

        impl Entry {

            /// Returns the citation key of this entry.
            #[must_use]
            pub fn cite(&self) -> &str {
                match self {
                    $(Self::$target(data) => &data.cite,)*
                }
            }

            /// Sets the citation key of this entry to a new value.
            pub fn set_cite(&mut self, cite: String) {
                match self {
                    $(Self::$target(data) => { data.cite = cite; },)*
                }
            }

            /// Returns the `title` field value of this entry.
            ///
            /// Each [`Entry`] type is required to have a `title` field so this should always
            /// represent a valid value.
            #[must_use]
            pub const fn title(&self) -> &QuotedString {
                match self {
                    $(Self::$target(data) => &data.title(),)*
                }
            }

            /// Returns the [`Field`]s of the entry.
            ///
            /// The fields returned include the required and optional fields in no particular
            /// order.
            #[must_use]
            pub fn fields(&self) -> Vec<Field> {
                match self {
                    $(Self::$target(data) => data.fields(),)*
                }
            }

            /// Searches for a field value that matches the `name` given.
            ///
            /// [`Self::find_field`] returns `Some(&QuotedString)` when a matching field is found
            /// and the return is the value of that matching field, returns `None` when no field
            /// matches the `name`.
            #[must_use]
            pub fn find_field(&self, name: &str) -> Option<&QuotedString> {
                match self {
                    $(Self::$target(data) => data.find_field(name),)*
                }
            }
        }

        impl FieldQuery for Entry {
            fn get_field(&self, name: &str) -> Option<&QuotedString> {
                match self {
                    $(Self::$target(data) => data.get_field(name),)*
                }
            }
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
                    $(
                        $(#[$req_comment])*
                        pub $req: QuotedString,
                    )+
                    /// Optional fields that a not essential for creating a valid entry of this
                    /// type.
                    pub optional: HashMap<String, QuotedString>,
                }

                impl $target {
                    /// Returns the `title` field value of this entry.
                    ///
                    /// This type is required to have a `title` field so this should always
                    /// represent a valid value.
                    #[must_use]
                    pub const fn title(&self) -> &QuotedString {
                        &self.title
                    }

                    /// Returns the [`Field`]s of the entry.
                    ///
                    /// The fields returned include the required and optional fields in no particular
                    /// order.
                    #[must_use]
                    pub fn fields(&self) -> Vec<Field> {
                        let mut fields: Vec<Field> = [$((stringify!($req), &self.$req),)+]
                            .into_iter()
                            .map(tuple_to_field)
                            .collect();
                        fields.extend(self.optional.iter().map(|(k, v)| tuple_to_field((k, v))));
                        fields
                    }

                    /// Searches for a field value that matches the `name` given.
                    ///
                    /// [`Self::find_field`] returns `Some(&QuotedString)` when a matching field is found
                    /// and the return is the value of that matching field, returns `None` when no field
                    /// matches the `name`.
                    #[must_use]
                    pub fn find_field(&self, name: &str) -> Option<&QuotedString> {
                        let normal_name = name.to_lowercase();
                        match normal_name.as_str() {
                            $(stringify!($req) => Some(&self.$req),)+
                            s => self.optional.get(s),
                        }
                    }

                    /// Creates a new [`Resolver`] for this type to ensure that the required fields
                    /// are set before the entry type can be built.
                    ///
                    /// Does not set the cite value of the resolver so will be generated based on
                    /// the field values.
                    #[must_use]
                    pub fn resolver() -> Resolver {
                        Resolver {
                            cite: None,
                            req: [$(Cow::Borrowed(stringify!($req)),)+].into_iter().collect(),
                            fields: HashMap::new(),
                            entry_resolve: resolve,
                        }
                    }

                    /// Creates a new [`Resolver`] for this type to ensure that the required fields
                    /// are set before the entry type can be built.
                    #[must_use]
                    pub fn resolver_with_cite<S: Into<String>>(cite: S) -> Resolver {
                        Resolver {
                            cite: Some(cite.into()),
                            req: [$(Cow::Borrowed(stringify!($req)),)+].into_iter().collect(),
                            fields: HashMap::new(),
                            entry_resolve: resolve,
                        }
                    }
                }

                impl FieldQuery for $target {
                    fn get_field(&self, name: &str) -> Option<&QuotedString> {
                        let normal_name = name.to_lowercase();
                        match normal_name.as_str() {
                            $(stringify!($req) => Some(&self.$req),)+
                            s => self.optional.get(s),
                        }
                    }
                }

                fn resolve(mut resolver: Resolver) -> Entry {
                    let cite = resolver.cite().to_string();

                    let data = $target {
                        cite,
                        $($req: resolver.fields.remove(stringify!($req)).unwrap(),)+
                        optional: resolver.fields,
                    };

                    Entry::$target(data)
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

                        resolver.set_field(field, QuotedString::new(c.to_string()));
                    }
                    resolver.set_field("test", QuotedString::new("value".to_owned()));
                    let res = resolver.resolve();
                    let entry = res.expect("All required fields added so should have built correctly");

                    let mut alpha = ('a'..).into_iter().map(|c| c.to_string());

                    if let Entry::$target(data) = entry {
                        $(
                            assert_eq!(alpha.next().unwrap(), &*data.$req);
                        )+
                        assert_eq!("value", &*data.optional["test"]);
                    } else {
                        panic!("Not the correct entry type!");
                    }
                }

            }
        )*
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
    other:
        /// Any other resource not supported by other entry variants.
        Other(
            /// Title of the resource.
            title
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