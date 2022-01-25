use crate::{
    api::format_api,
    ast::{Biblio, BiblioBuilder},
    format::BibTex,
};

use super::{Client, Error};

macro_rules! crossref_url {
    ($doi: ident) => {
        format!(
            "https://api.crossref.org/works/{}/transform/application/x-bibtex",
            $doi
        )
    };
}

#[inline]
pub(crate) fn get_entries_by_doi<C: Client>(
    doi: &str,
) -> Result<Result<Biblio, BiblioBuilder>, Error> {
    format_api::get_entry_by_url::<C, BibTex>(&crossref_url!(doi))
}

#[test]
fn crossref_url_macro_adds_doi_in_place() {
    let doi = "balloons";
    assert_eq!(
        "https://api.crossref.org/works/balloons/transform/application/x-bibtex",
        crossref_url!(doi)
    );
}
