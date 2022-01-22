use crate::{
    api::format_api,
    ast::{Biblio, BiblioBuilder},
    format::BibTex,
};

pub(crate) fn get_entry_by_rfc(number: usize) -> eyre::Result<Result<Biblio, BiblioBuilder>> {
    let url = format!("https://datatracker.ietf.org/doc/rfc{number}/bibtex/");
    format_api::get_entry_by_url::<BibTex>(&url)
}
