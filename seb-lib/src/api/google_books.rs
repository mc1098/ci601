use log::{info, trace};
use serde::Deserialize;

use crate::{
    ast::{self, Biblio, BiblioResolver, Resolver},
    Error, ErrorKind,
};

use super::Client;

const GOOGLE_BOOKS_URL: &str = "https://www.googleapis.com/books/v1/volumes?q=isbn:";

pub(crate) fn get_entries_by_isbn<C: Client>(
    isbn: &str,
) -> Result<std::result::Result<Biblio, BiblioResolver>, Error> {
    // remove hypen from ISBN-13 (if applicable)
    let isbn = isbn.replace('-', "");
    get_book_info::<C>(isbn)
        .and_then(Resolver::try_from)
        .map(|e| vec![e])
        .map(Biblio::try_resolve)
}

pub(crate) fn get_book_info<C: Client>(isbn: String) -> Result<Book, Error> {
    info!("Searching for ISBN '{isbn}' using Google Books API");
    let mut url = GOOGLE_BOOKS_URL.to_owned();
    url.push_str(&isbn);

    let client = C::default();
    let GoogleModel { mut items } = client.get_json(&url)?;

    trace!("Request was successful");

    let resolver = items
        .drain(..)
        .next()
        .ok_or_else(|| Error::new(ErrorKind::NoValue, "No books found!"))?;

    Ok(resolver.build(isbn))
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

impl TryFrom<Book> for Resolver {
    type Error = Error;

    fn try_from(book: Book) -> Result<Self, Error> {
        // Deconstruct book to take ownership of fields (avoids cloning).
        let Book {
            isbn,
            volume_info:
                VolumeInfo {
                    mut authors,
                    title,
                    publisher,
                    published_date,
                },
        } = book;

        let mut resolver = ast::Book::resolver();

        // date_parts = Year-Month-Day, where Day is not often used.
        let mut date_parts = published_date.split('-');

        let year = date_parts
            .next()
            .filter(|s| s.parse::<u16>().is_ok())
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::Deserialize,
                    "Date format was different then expected - aborting to avoid invalid dates in entry"
                )
            })?
            .to_owned();

        resolver.year(year);

        if let Some(month) = date_parts.next().filter(|s| s.parse::<u16>().is_ok()) {
            resolver.set_field("month", month);
        }

        resolver.title(title);

        authors.retain(|author| !author.is_empty());

        if !authors.is_empty() {
            resolver.author(authors.join(","));
        }

        resolver.publisher(publisher);
        resolver.set_field("isbn", isbn);

        Ok(resolver)
    }
}

#[cfg(test)]
mod tests {
    use super::{GoogleModel, Item, VolumeInfo};
    use crate::{
        api::{assert_url, impl_text_producer, MockClient},
        ast::{self, Resolver},
        Error, ErrorKind,
    };

    const GOOGLE_BOOK_JSON: &str = include_str!("../../tests/data/google_book_json.txt");

    impl_text_producer! {
        ValidJsonProducer => Ok(GOOGLE_BOOK_JSON.to_owned()),
        EmptyBookProducer => Ok(
            r#"{
                "items": []
            }"#.to_owned()
        ),
    }

    #[test]
    #[should_panic(expected = "No books found!")]
    fn no_items_in_json_returns_err_no_value() {
        let err = super::get_entries_by_isbn::<MockClient<EmptyBookProducer>>(&String::default());
        let kind = err.as_ref().map_err(Error::kind).map(|_| ());

        assert_eq!(Err(ErrorKind::NoValue), kind, "{:?}", err);
        drop(err.unwrap());
    }

    #[test]
    fn isbn_10_url_is_format_is_correct() {
        assert!(super::get_entries_by_isbn::<MockClient<ValidJsonProducer>>("0735619670").is_ok());
        assert_url!("https://www.googleapis.com/books/v1/volumes?q=isbn:0735619670");
    }

    #[test]
    fn isbn_13_url_is_format_is_correct() {
        assert!(
            super::get_entries_by_isbn::<MockClient<ValidJsonProducer>>("978-0380815937").is_ok()
        );
        // should strip the hypen in a ISBN-13 string
        assert_url!("https://www.googleapis.com/books/v1/volumes?q=isbn:9780380815937");
    }

    #[test]
    fn valid_json_produces_resolved_biblio() {
        let res = super::get_entries_by_isbn::<MockClient<ValidJsonProducer>>("test")
            .expect("ValidJsonProducer always produces a valid json String to be deserialized");

        let biblio = res.expect("Should produce a resolved Biblio");

        let entry = biblio
            .into_entries()
            .pop()
            .expect("Valid json should produce a single entry");

        assert_eq!("test", &**entry.get_field("isbn").unwrap());
        assert_eq!(ast::kind::Book, entry.kind());
    }

    #[test]
    #[should_panic(
        expected = "Date format was different then expected - aborting to avoid invalid dates in entry"
    )]
    fn invalid_date_format_returns_deserialize_error() {
        let ignore = "Ignore".to_owned();
        let item = Item {
            volume_info: VolumeInfo {
                authors: vec![ignore.clone()],
                title: ignore.clone(),
                publisher: ignore.clone(),
                published_date: "2022@apples".to_owned(),
            },
        };

        let book = item.build(ignore);
        let err = Resolver::try_from(book);

        let kind = err.as_ref().map(|_| ()).map_err(Error::kind);

        assert_eq!(Err(ErrorKind::Deserialize), kind);
        drop(err.unwrap());
    }

    #[test]
    fn published_date_with_year_and_month_parsed_correctly() {
        let item = Item {
            volume_info: VolumeInfo {
                authors: vec!["Ignore".to_owned()],
                title: "Ignore".to_owned(),
                publisher: "Ignore".to_owned(),
                published_date: "2002-09-01".to_owned(),
            },
        };

        let book = item.build("Ignore".to_owned());
        let entry: Box<dyn ast::EntryExt> = Resolver::try_from(book)
            .expect("Book is valid so will return a resolver")
            .resolve()
            .expect("Book should not fail to convert into an entry");

        assert_eq!(
            "2002",
            &**entry
                .get_field("year")
                .expect("Year field should be present")
        );
        assert_eq!(
            "09",
            &**entry
                .get_field("month")
                .expect("Month field should be present")
        );
    }

    #[test]
    fn book_can_be_derived_from_json() {
        let isbn = "0735619670";
        let mut model: GoogleModel = serde_json::from_str(GOOGLE_BOOK_JSON).unwrap();
        let book = &model.items.remove(0).build(isbn.to_owned());

        // ISBN is not in the response so will be the default value until changed.
        assert_eq!(isbn, book.isbn);
        assert_eq!("Steve McConnell", book.volume_info.authors[0]);
        assert_eq!("Code Complete", book.volume_info.title);
        assert_eq!("DV-Professional", book.volume_info.publisher);
        assert_eq!("2004", book.volume_info.published_date);
    }
}
