use biblatex::Entry;

use super::api::google_books;

pub(crate) fn get_book_by_isbn(isbn: &str) -> eyre::Result<Entry> {
    let book = google_books::get_book_info(isbn)?;
    Entry::try_from(book)
}
