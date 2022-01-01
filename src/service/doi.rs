use crate::{
    ast::Entry,
    format::{BibTex, Format},
};

use super::api::cross_ref;

use eyre::eyre;

pub(crate) fn get_entry_by_doi(doi: &str) -> eyre::Result<Entry> {
    let entry_info = cross_ref::get_entry_info_by_doi(doi)?;

    BibTex::new(entry_info)
        .parse()?
        .into_iter()
        .next()
        .ok_or_else(|| eyre!("Cannot create a valid reference from the request response"))
}
