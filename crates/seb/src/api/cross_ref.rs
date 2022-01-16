use eyre::Result;

use crate::{api::format_api, format::BibTex, Entry};

pub(crate) fn get_entries_by_doi(doi: &str) -> Result<Vec<Entry>> {
    let url = format!(
        "https://api.crossref.org/works/{}/transform/application/x-bibtex",
        doi
    );

    format_api::get_entry_by_url::<BibTex>(&url)
}
