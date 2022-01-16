use eyre::Result;

use crate::{api::format_api, format::BibTex, Entry};

pub(crate) fn get_entry_by_rfc(number: usize) -> Result<Vec<Entry>> {
    let url = format!("https://datatracker.ietf.org/doc/rfc{number}/bibtex/");
    format_api::get_entry_by_url::<BibTex>(&url)
}
