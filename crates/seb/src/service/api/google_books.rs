use eyre::{eyre, Context, Result};
use log::{info, trace};
use serde::Deserialize;

use crate::Entry;

const GOOGLE_BOOKS_URL: &str = "https:://www.googleapis.com/books/v1/volumes?q=isbn:";

pub(crate) fn get_entries_by_isbn(isbn: &str) -> Result<Vec<Entry>> {
    get_book_info(isbn)
        .and_then(Entry::try_from)
        .map(|e| vec![e])
}

pub(crate) fn get_book_info(isbn: &str) -> Result<Book> {
    info!("Searching for ISBN '{}' using Google Books API", isbn);
    let mut url = GOOGLE_BOOKS_URL.to_owned();
    url.push_str(isbn);

    let client = reqwest::blocking::Client::default();
    let GoogleModel { mut items } = client
        .get(&url)
        .send()
        .and_then(reqwest::blocking::Response::json)
        .wrap_err_with(|| eyre!("Cannot create valid reference for this ISBN"))?;

    trace!("Request was successful");

    let builder = items
        .drain(..)
        .next()
        .ok_or_else(|| eyre!("No Books found for ISBN of '{}', isbn"))?;

    Ok(builder.build(isbn.to_owned()))
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug))]
struct GoogleModel {
    items: Vec<Item>,
}

/// The API does not include the ISBN.. so this struct also acts as
/// a builder for the [`Book`] type, see [`VolumeInfo::build`].
#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug))]
struct Item {
    #[serde(rename = "volumeInfo")]
    volume_info: VolumeInfo,
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct Book {
    isbn: String,
    volume_info: VolumeInfo,
}

/// Volume information from the Google Book API
#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug))]
struct VolumeInfo {
    authors: Vec<String>,
    title: String,
    publisher: String,
    #[serde(rename = "publishedDate")]
    published_date: String,
}

impl Item {
    // We use a builder pattern here to enforce a valid [`Book`] is always returned.
    #[allow(clippy::missing_const_for_fn)] // can't be const
    fn build(self, isbn: String) -> Book {
        Book {
            isbn,
            volume_info: self.volume_info,
        }
    }
}

impl TryFrom<Book> for crate::ast::Entry {
    type Error = eyre::Report;

    fn try_from(book: Book) -> Result<Self> {
        // Deconstruct book to take ownership of fields (avoids cloning).
        let Book {
            isbn,
            volume_info:
                VolumeInfo {
                    mut authors,
                    title,
                    publisher,
                    published_date: year,
                },
        } = book;
        //
        // create citation_key based on first author + year.
        let mut cite = authors
            .drain(..)
            .next()
            .ok_or_else(|| eyre!("Not authors found from resource response"))?;
        cite.push_str(&year);

        let fields = [
            ("isbn", isbn),
            ("authors", authors.join(",")),
            ("title", title),
            ("publisher", publisher),
            ("year", year),
        ]
        .into_iter()
        .map(|(k, value)| crate::ast::Field {
            name: k.to_owned(),
            value,
        })
        .collect();

        Ok(Self {
            cite,
            variant: crate::ast::EntryType::Book,
            fields,
        })
    }
}

#[test]
fn book_can_be_derived_from_json() {
    let isbn = "0735619670";
    let json = include_str!("../../../../../tests/data/google_book_json.txt");
    let mut model: GoogleModel = serde_json::from_str(json).unwrap();
    let book = &model.items.remove(0).build(isbn.to_owned());

    // ISBN is not in the response so will be the default value until changed.
    assert_eq!(isbn, book.isbn);
    assert_eq!("Steve McConnell", book.volume_info.authors[0]);
    assert_eq!("Code Complete", book.volume_info.title);
    assert_eq!("DV-Professional", book.volume_info.publisher);
    assert_eq!("2004", book.volume_info.published_date);
}
