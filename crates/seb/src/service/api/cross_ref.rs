use eyre::{eyre, Context, Result};

use crate::{
    format::{BibTex, Format},
    Biblio, Entry,
};

pub(crate) fn get_entries_by_doi(doi: &str) -> Result<Vec<Entry>> {
    let url = format!(
        "https://api.crossref.org/works/{}/transform/application/x-bibtex",
        doi
    );

    let client = reqwest::blocking::Client::new();
    let bibtex = client
        .get(url)
        .send()
        .and_then(reqwest::blocking::Response::text)
        .map(BibTex::new)
        .wrap_err_with(|| eyre!("Cannot create valid reference for this doi"))?;

    bibtex.parse().map(Biblio::into_entries)
}
