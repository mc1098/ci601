use crate::ast::Entry;

use super::api::google_books;

/// Seek a bibliography entry using `ISBN`.
///
/// [`Entry`] returned by this function will also have a type of `book`.
///
/// # Errors
///
/// When the `ISBN` cannot be found by the service or if the resulting information cannot be used
/// to create a valid [`Entry`].
pub fn get_book_by_isbn(isbn: &str) -> eyre::Result<Entry> {
    google_books::get_book_info(isbn).and_then(Entry::try_from)
}
