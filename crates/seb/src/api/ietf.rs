use crate::{
    api::format_api,
    ast::{Biblio, BiblioResolver},
    format::BibTex,
};

use super::{Client, Error};

macro_rules! ietf_url {
    ($number: ident) => {
        format!("https://datatracker.ietf.org/doc/rfc{}/bibtex/", $number)
    };
}

pub(crate) fn get_entry_by_rfc<C: Client>(
    number: usize,
) -> Result<Result<Biblio, BiblioResolver>, Error> {
    format_api::get_entry_by_url::<C, BibTex>(&ietf_url!(number))
}

#[test]
fn ietf_url_macro_adds_number_in_place() {
    let number = 7230;
    assert_eq!(
        "https://datatracker.ietf.org/doc/rfc7230/bibtex/",
        ietf_url!(number)
    );
}
