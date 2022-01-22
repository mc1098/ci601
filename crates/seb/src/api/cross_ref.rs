use crate::{
    api::format_api,
    ast::{Biblio, BiblioBuilder},
    format::BibTex,
};

pub(crate) fn get_entries_by_doi(doi: &str) -> eyre::Result<Result<Biblio, BiblioBuilder>> {
    let url = format!(
        "https://api.crossref.org/works/{}/transform/application/x-bibtex",
        doi
    );

    format_api::get_entry_by_url::<BibTex>(&url)
}
