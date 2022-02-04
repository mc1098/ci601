use eyre::eyre;
use log::{info, trace};
use serde::Deserialize;

use crate::ast::{self, Biblio, BiblioResolver, Entry};

use super::{Client, Error};

const GOOGLE_BOOKS_URL: &str = "https://www.googleapis.com/books/v1/volumes?q=isbn:";

pub(crate) fn get_entries_by_isbn<C: Client>(
    isbn: &str,
) -> Result<std::result::Result<Biblio, BiblioResolver>, Error> {
    // remove hypen from ISBN-13 (if applicable)
    let isbn = isbn.replace('-', "");
    get_book_info::<C>(&isbn)
        .and_then(Entry::try_from)
        .map(|e| vec![e])
        .map(|entries| Ok(Biblio::new(entries)))
}

pub(crate) fn get_book_info<C: Client>(isbn: &str) -> Result<Book, Error> {
    info!("Searching for ISBN '{}' using Google Books API", isbn);
    let mut url = GOOGLE_BOOKS_URL.to_owned();
    url.push_str(isbn);

    let client = C::default();
    let GoogleModel { mut items } = client.get_json(&url)?;

    trace!("Request was successful");

    let resolver = items.drain(..).next().ok_or(Error::NoValue)?;

    Ok(resolver.build(isbn.to_owned()))
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug))]
struct GoogleModel {
    items: Vec<Item>,
}

/// The API does not include the ISBN.. so this struct also acts as
/// a resolver for the [`Book`] type, see [`VolumeInfo::build`].
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
    // We use a resolver pattern here to enforce a valid [`Book`] is always returned.
    #[allow(clippy::missing_const_for_fn)] // can't be const
    fn build(self, isbn: String) -> Book {
        Book {
            isbn,
            volume_info: self.volume_info,
        }
    }
}

impl TryFrom<Book> for Entry {
    type Error = Error;

    fn try_from(book: Book) -> Result<Self, Error> {
        // Deconstruct book to take ownership of fields (avoids cloning).
        let Book {
            isbn,
            volume_info:
                VolumeInfo {
                    authors,
                    title,
                    publisher,
                    published_date: year,
                },
        } = book;

        // create citation_key based on first author + year.
        let mut cite = authors
            .get(0)
            .cloned()
            .map(|mut s| {
                s.retain(|c| !c.is_whitespace());
                s
            })
            .ok_or_else(|| {
                Error::Deserialize(eyre!("Not authors found from resource response").into())
            })?;
        cite.push_str(&year);

        let title = ast::QuotedString::new(title);

        let data = ast::Book {
            cite,
            author: ast::QuotedString::new(authors.join(",")),
            title,
            publisher: ast::QuotedString::new(publisher),
            year: ast::QuotedString::new(year),
            optional: std::collections::HashMap::from([(
                "isbn".to_owned(),
                ast::QuotedString::new(isbn),
            )]),
        };

        Ok(Self::Book(data))
    }
}

#[cfg(test)]
mod tests {
    use crate::api::{impl_text_producer, MockJsonClient};

    impl_text_producer! {
        ValidJsonProducer => Ok(include_str!("../../../../tests/data/google_book_json.txt").to_owned()),
    }

    #[test]
    fn valid_json_produces_resolved_biblio() {
        let res = super::get_entries_by_isbn::<MockJsonClient<ValidJsonProducer>>("test")
            .expect("ValidJsonProducer always produces a valid json String to be deserialized");

        res.expect("Should produce a resolved Biblio");
    }
}

#[test]
fn book_can_be_derived_from_json() {
    let isbn = "0735619670";
    let json = include_str!("../../../../tests/data/google_book_json.txt");
    let mut model: GoogleModel = serde_json::from_str(json).unwrap();
    let book = &model.items.remove(0).build(isbn.to_owned());

    // ISBN is not in the response so will be the default value until changed.
    assert_eq!(isbn, book.isbn);
    assert_eq!("Steve McConnell", book.volume_info.authors[0]);
    assert_eq!("Code Complete", book.volume_info.title);
    assert_eq!("DV-Professional", book.volume_info.publisher);
    assert_eq!("2004", book.volume_info.published_date);
}
