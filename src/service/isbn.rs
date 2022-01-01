use crate::ast::Entry;

use super::api::google_books;

pub(crate) fn get_book_by_isbn(isbn: &str) -> eyre::Result<Entry> {
    google_books::get_book_info(isbn).and_then(Entry::try_from)
}
