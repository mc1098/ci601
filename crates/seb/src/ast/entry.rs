use std::{borrow::Cow, collections::HashMap};

use super::{Field, QuotedString};

/// An intermediate representation of a bibliography entry which is not tied to a specific end
/// format.
#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Clone))]
pub struct Entry {
    pub citation_key: String,
    pub entry_data: EntryData,
}

impl Entry {
    #[must_use]
    pub const fn title(&self) -> &QuotedString {
        self.entry_data.title()
    }

    #[must_use]
    pub fn fields(&self) -> Vec<Field> {
        self.entry_data.fields()
    }

    #[must_use]
    pub fn contains_field(&self, name: &str) -> bool {
        self.find_field(name).is_some()
    }

    #[must_use]
    pub fn find_field(&self, name: &str) -> Option<&QuotedString> {
        self.entry_data.find_field(name)
    }
}

const fn tuple_to_field<'name, 'value>(
    (name, value): (&'name str, &'value QuotedString),
) -> Field<'name, 'value> {
    Field {
        name: Cow::Borrowed(name),
        value: Cow::Borrowed(value),
    }
}

macro_rules! impl_builder {
    ($($mod:ident: $target:ident($($req:ident),+)),* $(,)?)=> {
        #[derive(Debug, PartialEq)]
        #[cfg_attr(test, derive(Clone))]
        pub enum EntryData {
            $($target($target),)*
        }

        impl EntryData {
            #[must_use]
            pub const fn title(&self) -> &QuotedString {
                match self {
                    $(Self::$target(data) => &data.title(),)*
                }
            }

            #[must_use]
            pub fn fields(&self) -> Vec<Field> {
                match self {
                    $(Self::$target(data) => data.fields(),)*
                }
            }

            #[must_use]
            pub fn find_field(&self, name: &str) -> Option<&QuotedString> {
                match self {
                    $(Self::$target(data) => data.find_field(name),)*
                }
            }
        }
        $(
            pub use $mod::*;
            mod $mod {
                use super::*;

                #[derive(Debug, PartialEq)]
                #[cfg_attr(test, derive(Clone))]
                pub struct $target {
                    $(pub $req: QuotedString,)+
                    pub optional: HashMap<String, QuotedString>,
                }

                impl $target {
                    #[must_use]
                    pub const fn title(&self) -> &QuotedString {
                        &self.title
                    }

                    #[must_use]
                    pub fn fields(&self) -> Vec<Field> {
                        let mut fields: Vec<Field> = [$((stringify!($req), &self.$req),)+]
                            .into_iter()
                            .map(tuple_to_field)
                            .collect();
                        fields.extend(self.optional.iter().map(|(k, v)| tuple_to_field((k, v))));
                        fields
                    }

                    #[must_use]
                    pub fn find_field(&self, name: &str) -> Option<&QuotedString> {
                        let normal_name = name.to_lowercase();
                        match normal_name.as_str() {
                            $(stringify!($req) => Some(&self.$req),)+
                            s => self.optional.get(s),
                        }
                    }

                    #[must_use]
                    pub fn builder() -> Builder {
                        Builder {
                            req: [$(stringify!($req),)+].into_iter().collect(),
                            fields: HashMap::new(),
                            entry_build: build,
                        }
                    }
                }

                fn build(mut builder: Builder) -> EntryData {
                    let data = $target {
                        $($req: builder.fields.remove(stringify!($req)).unwrap(),)+
                        optional: builder.fields,
                    };

                    EntryData::$target(data)
                }

                #[test]
                fn builder_only_returns_ok_when_all_required_fields() {
                    use std::collections::VecDeque;

                    let mut builder = $target::builder();
                    let mut required: VecDeque<_> = [$(stringify!($req),)+].into_iter().collect();

                    let iter = std::iter::from_fn(move || required.pop_front());
                    let iter = iter.zip(('a'..).into_iter());

                    for (field, c) in iter {
                        builder = builder
                            .build()
                            .expect_err("Builder should not build correctly without required fields");

                        builder.set_field(field, QuotedString::new(c.to_string()));
                    }
                    builder.set_field("test", QuotedString::new("value".to_owned()));
                    let res = builder.build();
                    let entry_data = res.expect("All required fields added so should have built correctly");

                    let mut alpha = ('a'..).into_iter().map(|c| c.to_string());

                    if let EntryData::$target(data) = entry_data {
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

impl_builder! {
    article: Article(author, title, journal, year),
    book: Book(author, title, publisher, year),
    booklet: Booklet(title),
    //inbook
    book_chapter: BookChapter(author, title, chapter, publisher, year),
    book_pages: BookPages(author, title, pages, publisher, year),
    book_section: BookSection(author, title, book_title, publisher, year),
    in_proceedings: InProceedings(author, title, book_title, year),
    manual: Manual(title),
    master_thesis: MasterThesis(author, title, school, year),
    phd_thesis: PhdThesis(author, title, school, year),
    other: Other(title),
    proceedings: Proceedings(title, year),
    tech_report: TechReport(author, title, institution, year),
    unpublished: Unpublished(author, title),
}

#[derive(Debug)]
pub struct Builder {
    req: Vec<&'static str>,
    fields: HashMap<String, QuotedString>,
    entry_build: fn(Self) -> EntryData,
}

impl Builder {
    /// Build an entry from the fields added in this builder.
    ///
    /// # Errors
    /// Returns `Err(Self)` when the required fields have not been set to make a valid [`Entry`],
    /// returning `Self` allows for the user to retry.
    pub fn build(self) -> Result<EntryData, Self> {
        if self.req.iter().all(|r| self.fields.contains_key(*r)) {
            Ok((self.entry_build)(self))
        } else {
            Err(self)
        }
    }

    #[must_use]
    pub fn required_fields(&self) -> &[&str] {
        &self.req
    }

    pub fn set_field(&mut self, name: &str, value: QuotedString) {
        self.fields.insert(name.to_lowercase(), value);
    }
}
