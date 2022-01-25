use crate::{
    ast::{Biblio, BiblioBuilder},
    format::Format,
};

use super::{Client, Error};

pub(crate) fn get_entry_by_url<C: Client, F: Format>(
    url: &str,
) -> Result<Result<Biblio, BiblioBuilder>, Error> {
    let client = C::default();

    client
        .get_text(url)
        .map(F::new)
        .and_then(|f| f.parse().map_err(|r| Error::Deserialize(r.into())))
}

#[cfg(test)]
mod tests {

    use crate::{
        api::{impl_text_producer, MockTextClient, NetworkErrorProducer},
        format::BibTex,
    };

    use super::{get_entry_by_url, Error};

    impl_text_producer! {
        NotBibTexProducer => Ok("This is not valid BibTeX".to_owned()),
        ValidIncompleteEntryProducer => Ok("@book{test, author={Me}, publisher={Also me},}".to_owned()),
        ValidCompleteEntryProducer => Ok("@manual{cite, title={This is a title},}".to_owned()),
    }

    #[test]
    fn client_text_error() {
        let err = get_entry_by_url::<MockTextClient<NetworkErrorProducer>, BibTex>("test")
            .expect_err("MockErrorClient should always cause an error");

        assert!(matches!(err, Error::Network(_)));
    }

    #[test]
    fn client_text_parse_error() {
        let err = get_entry_by_url::<MockTextClient<NotBibTexProducer>, BibTex>("test")
            .expect_err("MockErrorClient should always cause an error");

        assert!(matches!(err, Error::Deserialize(_)));
    }

    #[test]
    fn valid_and_incomplete_entry_returns_builder() {
        let should_be_builder =
            get_entry_by_url::<MockTextClient<ValidIncompleteEntryProducer>, BibTex>("test")
                .expect("ValidIncompleteEntryProducer should always produce an ok response");

        let mut builder = should_be_builder
            .expect_err("ValidIncompleteEntryProducer should produce a valid but incomplete entry");

        let entry_builder = builder
            .unresolved()
            .next()
            .expect("BiblioBuilder should have a single unresolved entry builder");

        assert_eq!("test", entry_builder.cite());
    }

    #[test]
    fn valid_and_complete_entry_returns_biblio() {
        let should_be_biblio =
            get_entry_by_url::<MockTextClient<ValidCompleteEntryProducer>, BibTex>("test")
                .expect("ValidCompleteEntryProducer should always produce an ok response");

        let biblio = should_be_biblio
            .expect("ValidCompleteEntryProducer should produce a valid and complete entry");

        let entry = biblio.into_entries().remove(0);

        assert_eq!("cite", entry.cite());
        assert_eq!("This is a title", &**entry.title());
    }
}
