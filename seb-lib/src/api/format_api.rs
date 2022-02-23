use crate::{
    ast::{Biblio, BiblioResolver},
    format::Format,
    Error, ErrorKind,
};

use super::Client;

pub(crate) fn get_entry_by_url<C: Client, F: Format>(
    url: &str,
) -> Result<Result<Biblio, BiblioResolver>, Error> {
    let client = C::default();

    client
        .get_text(url)
        .and_then(|text| {
            if text.is_empty() {
                Err(Error::new(
                    ErrorKind::NoValue,
                    "Request did not find any results",
                ))
            } else {
                Ok(F::new(text))
            }
        })
        .and_then(Format::parse)
}

#[cfg(test)]
mod tests {

    use crate::{
        api::{impl_text_producer, MockClient, NetworkErrorProducer},
        ast::EntryExt,
        format::BibTex,
        ErrorKind,
    };

    use super::get_entry_by_url;

    impl_text_producer! {
        NotBibTexProducer => Ok("This is not valid BibTeX".to_owned()),
        ValidIncompleteEntryProducer => Ok("@book{test, author={Me}, publisher={Also me},}".to_owned()),
        ValidCompleteEntryProducer => Ok("@manual{cite, title={This is a title},}".to_owned()),
    }

    #[test]
    fn client_text_error() {
        let err = get_entry_by_url::<MockClient<NetworkErrorProducer>, BibTex>("test")
            .expect_err("MockErrorClient should always cause an error");

        assert_eq!(ErrorKind::IO, err.kind());
    }

    #[test]
    fn client_text_parse_error() {
        let err = get_entry_by_url::<MockClient<NotBibTexProducer>, BibTex>("test")
            .expect_err("MockErrorClient should always cause an error");

        assert_eq!(ErrorKind::Deserialize, err.kind());
    }

    #[test]
    fn valid_and_incomplete_entry_returns_resolver() {
        let should_be_resolver =
            get_entry_by_url::<MockClient<ValidIncompleteEntryProducer>, BibTex>("test")
                .expect("ValidIncompleteEntryProducer should always produce an ok response");

        let mut resolver = should_be_resolver
            .expect_err("ValidIncompleteEntryProducer should produce a valid but incomplete entry");

        let entry_resolver = resolver
            .unresolved()
            .next()
            .expect("BiblioResolver should have a single unresolved entry resolver");

        assert_eq!("test", entry_resolver.cite());
    }

    #[test]
    fn valid_and_complete_entry_returns_biblio() {
        let should_be_biblio =
            get_entry_by_url::<MockClient<ValidCompleteEntryProducer>, BibTex>("test")
                .expect("ValidCompleteEntryProducer should always produce an ok response");

        let biblio = should_be_biblio
            .expect("ValidCompleteEntryProducer should produce a valid and complete entry");

        let entry = biblio.into_entries().remove(0);

        assert_eq!("cite", entry.cite());
        assert_eq!("This is a title", &**entry.title());
    }
}
