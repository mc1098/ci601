use eyre::{eyre, Context};

pub(in crate::service) fn get_entry_info_by_doi(doi: &str) -> eyre::Result<String> {
    let url = format!(
        "https://api.crossref.org/works/{}/transform/application/x-bibtex",
        doi
    );

    let client = reqwest::blocking::Client::new();
    client
        .get(url)
        .send()
        .and_then(|r| r.text())
        .wrap_err_with(|| eyre!("Cannot create valid reference for this doi"))
}
