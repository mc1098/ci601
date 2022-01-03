use crate::{
    ast::Entry,
    format::{BibTex, Format},
};

use super::api::cross_ref;

use eyre::eyre;

/// Seek a bibliography entry using a `doi`.
///
/// # Errors
///
/// When the `doi` cannot be found by the service or if the resulting information cannot be used
/// to create a valid [`Entry`].
pub fn get_entry_by_doi(doi: &str) -> eyre::Result<Entry> {
    let entry_info = cross_ref::get_entry_info_by_doi(doi)?;

    BibTex::new(entry_info)
        .parse()?
        .into_iter()
        .next()
        .ok_or_else(|| eyre!("Cannot create a valid reference from the request response"))
}
