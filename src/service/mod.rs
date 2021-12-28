mod api;
mod doi;
mod isbn;

pub(crate) use doi::get_entry_by_doi;
pub(crate) use isbn::get_book_by_isbn;
