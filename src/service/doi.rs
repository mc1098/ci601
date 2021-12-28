use super::api::cross_ref;

use biblatex::{Bibliography, Entry};
use eyre::eyre;

pub(crate) fn get_entry_by_doi(doi: &str) -> eyre::Result<Entry> {
    let entry_info = cross_ref::get_entry_info_by_doi(doi)?;

    Bibliography::parse(&entry_info)
        .and_then(|b| b.into_iter().next())
        .ok_or_else(|| eyre!("Cannot create a valid reference from the request response"))
}
