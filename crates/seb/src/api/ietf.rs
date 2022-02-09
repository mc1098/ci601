use crate::{
    api::format_api,
    ast::{Biblio, BiblioResolver},
    format::BibTex,
    Error,
};

use super::Client;

pub(crate) fn get_entry_by_rfc<C: Client>(
    number: usize,
) -> Result<Result<Biblio, BiblioResolver>, Error> {
    let url = format!("https://datatracker.ietf.org/doc/rfc{number}/bibtex");
    format_api::get_entry_by_url::<C, BibTex>(&url)
}

#[cfg(test)]
mod tests {
    use crate::{
        api::{assert_url, MockClient},
        ErrorKind,
    };

    #[test]
    fn error_no_value_on_empty_text() {
        let err = super::get_entry_by_rfc::<MockClient>(7230)
            .expect_err("Empty text should cause an error");
        assert_eq!(ErrorKind::NoValue, err.kind());
    }

    #[test]
    fn url_format_is_correct() {
        assert!(super::get_entry_by_rfc::<MockClient>(7230).is_err());
        assert_url!("https://datatracker.ietf.org/doc/rfc7230/bibtex");
    }
}
