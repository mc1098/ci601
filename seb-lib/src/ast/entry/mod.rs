use std::{borrow::Cow, collections::HashMap};

use super::{Field, QuotedString};

mod resolver;

pub use resolver::*;

/// Trait for querying data structures with fields.
pub trait FieldQuery {
    /// Searches for a field value that matches the `name` given.
    ///
    /// [`Self::get_field`] returns `Some(&QuotedString)` when a matching field is found
    /// and the return is the value of that matching field, returns `None` when no field
    /// matches the `name`.
    fn get_field(&self, name: &str) -> Option<&QuotedString>;
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
            /// Any other resource not supported by other entry variants.
            Other(Other),
        }

        /// Types of bibliographic entries
        #[derive(Debug, Clone, PartialEq)]
        pub enum EntryKind<'entry> {
            $(
                $(#[$target_comment])*
                $target,
            )*
            /// Custom entry type.
            Other(Cow<'entry, str>),
        }

        impl EntryKind<'_> {
            /// Returns a slice of the required fields that need to be set in order to make this
            /// entry kind valid.
            #[must_use]
            pub const fn required_fields(&self) -> &'static [&'static str] {
                match self {
                    $(Self::$target => &[$(stringify!($req),)+],)*
                    Self::Other(_) => &["title"],
                }
            }
        }

        impl std::fmt::Display for EntryKind<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                match self {
                    $(Self::$target => write!(f, stringify!($mod)),)*
                    Self::Other(s) => write!(f, "{s}"),
                }
            }
        }

        impl Entry {

            /// Returns the type of Entry
            ///
            /// This can be used instead of the full entry enum for deciding to perform some action
            /// based on the type of entry.
            #[must_use]
            pub fn kind(&self) -> EntryKind<'_> {
                match self {
                    $(Self::$target(_) => EntryKind::$target,)*
                    Self::Other(other) => EntryKind::Other(Cow::Borrowed(other.kind())),
                }
            }

            /// Returns the citation key of this entry.
            #[must_use]
            pub fn cite(&self) -> &str {
                match self {
                    $(Self::$target(data) => &data.cite,)*
                    Self::Other(other) => &other.cite,
                }
            }

            /// Sets the citation key of this entry to a new value.
            pub fn set_cite(&mut self, cite: String) {
                match self {
                    $(Self::$target(data) => { data.cite = cite; },)*
                    Self::Other(data) => { data.cite = cite; },
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
                    Self::Other(data) => &data.title,
                }
            }

            /// Returns the [`Field`]s of the entry.
            ///
            /// The fields returned include the required and optional fields in no particular
            /// order.
            #[must_use]
            pub fn fields(&self) -> Vec<Field<'_>> {
                match self {
                    $(Self::$target(data) => data.fields(),)*
                    Self::Other(data) => data.fields(),
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
                    Self::Other(data) => data.find_field(name),
                }
            }

            /// Creates a new [`Resolver`] for this type to ensure that the required fields
            /// are set before the entry type can be built.
            ///
            /// Does not set the cite value of the resolver so will be generated based on
            /// the field values.
            #[must_use]
            pub fn resolver(kind: EntryKind<'_>) -> Resolver {
                match kind {
                    $(EntryKind::$target => $mod::$target::resolver(),)*
                    EntryKind::Other(Cow::Owned(s)) => Other::resolver(s),
                    EntryKind::Other(Cow::Borrowed(s)) => Other::resolver(s.to_owned()),
                }
            }

            /// Creates a new [`Resolver`] for this type to ensure that the required fields
            /// are set before the entry type can be built.
            #[must_use]
            pub fn resolver_with_cite<S: Into<String>>(kind: EntryKind<'_>, cite: S) -> Resolver {
                match kind {
                    $(EntryKind::$target => $mod::$target::resolver_with_cite(cite),)*
                    EntryKind::Other(Cow::Owned(s)) => Other::resolver_with_cite(s, cite),
                    EntryKind::Other(Cow::Borrowed(s)) => Other::resolver_with_cite(s.to_owned(), cite),
                }
            }
        }

        impl FieldQuery for Entry {
            fn get_field(&self, name: &str) -> Option<&QuotedString> {
                match self {
                    $(Self::$target(data) => data.get_field(name),)*
                    Self::Other(data) => data.get_field(name),
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
                    pub fn fields(&self) -> Vec<Field<'_>> {
                        let mut fields: Vec<_> = [$((stringify!($req), &self.$req),)+]
                            .into_iter()
                            .map(Field::from)
                            .collect();
                        fields.extend(self.optional.iter().map(Field::from));
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
                    pub(super) fn resolver() -> Resolver {
                        Resolver::new(EntryKind::$target, None, resolve)
                    }

                    /// Creates a new [`Resolver`] for this type to ensure that the required fields
                    /// are set before the entry type can be built.
                    #[must_use]
                    pub(super) fn resolver_with_cite<S: Into<String>>(cite: S) -> Resolver {
                        Resolver::new(EntryKind::$target, Some(cite.into()), resolve)
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

/// Any other resource not supported by other entry variants.
#[derive(Clone, Debug, PartialEq)]
pub struct Other {
    cite: String,
    kind: String,
    title: QuotedString,
    optional: HashMap<String, QuotedString>,
}

impl Other {
    /// The type of this custom entry.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    #[must_use]
    fn resolver(kind: String) -> Resolver {
        Resolver {
            target: EntryKind::Other(kind.into()),
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
            target: EntryKind::Other(kind.into()),
            cite: Some(cite.into()),
            req: vec!["title"],
            fields: HashMap::new(),
            entry_resolve: Self::resolve,
        }
    }

    fn resolve(mut resolver: Resolver) -> Entry {
        Entry::Other(Other {
            cite: resolver.cite().to_string(),
            kind: resolver.target.to_string(),
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
    /// Searches for a field value that matches the `name` given.
    ///
    /// [`Self::find_field`] returns `Some(&QuotedString)` when a matching field is found
    /// and the return is the value of that matching field, returns `None` when no field
    /// matches the `name`.
    #[must_use]
    pub fn find_field(&self, name: &str) -> Option<&QuotedString> {
        let normal_name = name.to_lowercase();
        match normal_name.as_str() {
            "title" => Some(&self.title),
            s => self.optional.get(s),
        }
    }
}

impl FieldQuery for Other {
    fn get_field(&self, name: &str) -> Option<&QuotedString> {
        self.find_field(name)
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
