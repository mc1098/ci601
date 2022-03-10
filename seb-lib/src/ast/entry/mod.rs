use std::{borrow::Cow, collections::HashMap, fmt::Debug};

use super::{Field, QuotedString};

mod resolver;

pub use resolver::*;
use seb_macro::Entry;

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

macro_rules! entry_structs {
    ($(
        $(#[$entry_comment:meta])*
        $entry: ident {
            $(
                $(#[$field_comment:meta])+
                $req:ident,
            )*
        }
    )*) => {
        $(

            $(#[$entry_comment])*
            #[derive(Clone, Debug, Entry, PartialEq)]
            pub struct $entry {
                cite: String,
                optional: HashMap<String, QuotedString>,
                $($req: QuotedString,)*
            }
        )*
    };

}

entry_structs! {
    /// An article entry type
    Article {
        /// Authors of the article.
        author,
        /// Title of the article.
        title,
        /// The journal that contains this article.
        journal,
        /// The year of this article.
        year,
    }

    /// The book entry type
    Book {
        /// Authors of the book.
        author,
        /// Title of the book.
        title,
        /// Publisher of the book.
        publisher,
        /// Year the book was published.
        year,
    }

    /// The booklet entry type.
    Booklet {
        /// Title of the booklet.
        title,
    }

    //inbook
    /// A chapter of a book
    BookChapter {
        /// Authors of the book.
        author,
        /// Title of the book.
        title,
        /// Name of the chapter.
        chapter,
        /// Publisher of the book.
        publisher,
        /// Year the book was published.
        year,
    }

    /// A page range of a book
    BookPages {
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
        year,
    }

    /// A section of a book with a title.
    BookSection {
        /// Authors of the book.
        author,
        /// Title of the section.
        title,
        /// Title of the book.
        book_title,
        /// Publisher of the book.
        publisher,
        /// Year the book was published.
        year,
    }

    /// Published paper in a conference proceedings.
    InProceedings {
        /// Authors of the book.
        author,
        /// Title of the conference.
        title,
        /// Title of the paper.
        book_title,
        /// Year the paper was published.
        year,
    }

    /// Manual for technical information for machine software.
    Manual {
        /// Title of the manual.
        title,
    }

    /// A thesis for a Master's level degree.
    MasterThesis {
        /// Authors of the thesis.
        author,
        /// Title of the thesis.
        title,
        /// School of the author.
        school,
        /// Year the paper was published.
        year,
    }

    /// A thesis for a PhD level degree.
    PhdThesis {
        /// Authors of the thesis.
        author,
        /// Title of the thesis.
        title,
        /// School of the author.
        school,
        /// Year the paper was published.
        year,
    }

    /// A conference proceeding.
    Proceedings {
        /// Title of the conference.
        title,
        /// Year of the conference.
        year,
    }

    /// A technical report.
    TechReport {
        /// Authors of the report.
        author,
        /// Title of the report.
        title,
        /// Institution that published the report.
        institution,
        /// Year of the report.
        year,
    }

    /// A document that has not been officially published.
    Unpublished {
        /// Authors of the document.
        author,
        /// Title of the document.
        title,
    }

}

#[derive(Clone, Debug, Entry, PartialEq)]
/// A catch all type for not supported entry types.
pub struct Other {
    cite: String,
    #[kind]
    kind: Cow<'static, str>,
    title: QuotedString,
    optional: HashMap<String, QuotedString>,
}
