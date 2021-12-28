use biblatex::{Chunk, Entry, Person};
use eyre::{eyre, Context};
use log::*;
use serde::Deserialize;

type Result<T> = eyre::Result<T>;

const GOOGLE_BOOKS_URL: &str = "https:://www.googleapis.com/books/v1/volumes?q=isbn:";

pub(crate) fn get_book_info(isbn: &str) -> Result<Book> {
    info!("Searching for ISBN '{}' using Google Books API", isbn);
    let mut url = GOOGLE_BOOKS_URL.to_owned();
    url.push_str(isbn);

    let client = reqwest::blocking::Client::default();
    let GoogleModel { mut items } = client
        .get(&url)
        .send()
        .and_then(|r| r.json())
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
    items: Vec<VolumeInfo>,
}

pub(crate) struct Book {
    isbn: String,
    volume_info: VolumeInfo,
}

/// Volume information from the Google Book API
///
/// The API does not include the ISBN.. so this struct also acts as
/// a builder for the [`Book`] type, see [`VolumeInfo::build`].
#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug))]
struct VolumeInfo {
    authors: Vec<String>,
    title: String,
    publisher: String,
    #[serde(rename = "publishedDate")]
    published_date: String,
}

impl VolumeInfo {
    // We use a builder pattern here to enforce a valid [`Book`] is always returned.
    fn build(self, isbn: String) -> Book {
        Book {
            isbn,
            volume_info: self,
        }
    }
}

impl TryFrom<Book> for Entry {
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

        // create citation_key based on first author + year.
        let mut citation_key = authors
            .drain(..)
            .next()
            .ok_or_else(|| eyre!("Not authors found from resource response"))?;
        citation_key.push_str(&year);

        let mut entry = Entry::new(citation_key.to_owned(), biblatex::EntryType::Book);

        let authors = authors
            .into_iter()
            .map(Chunk::Normal)
            .map(|c| Person::parse(&[c]))
            .collect();
        entry.set_author(authors);

        entry.set_title(vec![Chunk::Normal(title)]);
        entry.set_publisher(vec![vec![Chunk::Normal(publisher)]]);

        let date = biblatex::Date::parse_three_fields(&[Chunk::Normal(year)], None, None)
            .ok_or_else(|| eyre!("Date has an invalid format in resource response"))?;
        entry.set_date(date);
        entry.set_isbn(vec![Chunk::Normal(isbn)]);

        Ok(entry)
    }
}

#[test]
fn book_can_be_derived_from_json() {
    let json = include_str!("../../../tests/data/google_book_json.txt");
    let mut model: GoogleModel = serde_json::from_str(json).unwrap();
    let book = &model.items.remove(0).build("0735619670".to_owned());

    // ISBN is not in the response so will be the default value until changed.
    assert_eq!(String::new(), book.isbn);
    assert_eq!("Steve McConnell", book.volume_info.authors[0]);
    assert_eq!("Code Complete", book.volume_info.title);
    assert_eq!("DV-Professional", book.volume_info.publisher);
    assert_eq!("2004", book.volume_info.published_date);
}
