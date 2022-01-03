use eyre::{eyre, Context};

pub(crate) fn get_entry_info_by_doi(doi: &str) -> eyre::Result<String> {
    let url = format!(
        "https://api.crossref.org/works/{}/transform/application/x-bibtex",
        doi
    );

    let client = reqwest::blocking::Client::new();
    client
        .get(url)
        .send()
        .and_then(reqwest::blocking::Response::text)
        .wrap_err_with(|| eyre!("Cannot create valid reference for this doi"))
}
