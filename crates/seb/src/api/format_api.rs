use crate::{
    ast::{Biblio, Entry},
    format::Format,
};

use eyre::{eyre, Context, Result};

pub(crate) fn get_entry_by_url<F: Format>(url: &str) -> Result<Vec<Entry>> {
    let client = reqwest::blocking::Client::new();
    client
        .get(url)
        .send()
        .and_then(reqwest::blocking::Response::text)
        .map(F::new)
        .wrap_err_with(|| {
            eyre!(
                "There was a problem in retrieving the text from the url: '{}'",
                url
            )
        })
        .and_then(Format::parse)
        .map(Biblio::into_entries)
        .and_then(|entries| {
            if entries.is_empty() {
                Err(eyre!("No entry found at the url: `{}`"))
            } else {
                Ok(entries)
            }
        })
}
