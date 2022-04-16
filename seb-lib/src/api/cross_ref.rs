use serde::Deserialize;

use crate::{
    api::format_api,
    ast::{Biblio, BiblioResolver},
    format::BibTex,
    Error, ErrorKind,
};

use super::Client;

#[inline]
pub(crate) fn get_entries_by_doi<C: Client>(
    doi: &str,
) -> Result<Result<Biblio, BiblioResolver>, Error> {
    let url = format!("https://api.crossref.org/works/{doi}/transform/application/x-bibtex");
    format_api::get_entry_by_url::<C, BibTex>(&url)
}

#[derive(Deserialize)]
struct QueryResult {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    items: Vec<EntryStub>,
}

#[derive(Deserialize)]
struct EntryStub {
    #[serde(rename = "DOI")]
    doi: String,
    title: Vec<String>,
}

impl EntryStub {
    fn into_tuple(mut self) -> (String, String) {
        (self.doi, self.title.remove(0))
    }
}

pub(crate) fn get_entry_stubs_by_title<C: Client>(
    title: &str,
) -> Result<Vec<(String, String)>, Error> {
    let url = format!("https://api.crossref.org/works?query.title={title}&select=DOI,title");
    let client = C::default();

    let query_result: QueryResult = client.get_json(&url)?;
    let items = query_result.message.items;
    // check for empty array of items
    if items.is_empty() {
        Err(Error::new(
            ErrorKind::NoValue,
            format!("No entries found with a title of {title}"),
        ))
    } else {
        Ok(items.into_iter().map(EntryStub::into_tuple).collect())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        api::{assert_url, impl_text_producer, MockClient},
        ErrorKind,
    };

    use super::QueryResult;

    const ENTRY_STUB_JSON: &str = include_str!("../../tests/data/crossref_entry_stub.json");

    #[test]
    fn by_doi_url_format_is_correct() {
        assert!(super::get_entries_by_doi::<MockClient>("balloons").is_err());
        assert_url!("https://api.crossref.org/works/balloons/transform/application/x-bibtex");
    }

    #[test]
    fn json_can_be_deserialized_to_query_result() {
        let qr: QueryResult = serde_json::from_str(ENTRY_STUB_JSON).unwrap();
        assert_eq!(20, qr.message.items.len());
    }

    impl_text_producer! {
        ValidJsonProducer => Ok(ENTRY_STUB_JSON.to_owned()),
        EmptyItemProducer => Ok(
            r#"{
                "message": {
                    "items": []
                }
            }"#.to_owned()
        ),
    }

    #[test]
    fn valid_json_produces_resolved_biblio() {
        let res = super::get_entry_stubs_by_title::<MockClient<ValidJsonProducer>>("test")
            .expect("ValidJsonProducer always produces a valid json String to be deserialized");

        assert_eq!(20, res.len());
    }

    type EmptyItemClient = MockClient<EmptyItemProducer>;

    #[test]
    fn by_title_url_format_is_correct() {
        assert!(super::get_entry_stubs_by_title::<EmptyItemClient>("My test title").is_err());
        // Not expecting percent encoding here, the str to URL conversion will do this.
        assert_url!("https://api.crossref.org/works?query.title=My test title&select=DOI,title");
    }

    #[test]
    fn empty_item_returns_no_value_error() {
        let res = super::get_entry_stubs_by_title::<EmptyItemClient>("test")
            .expect_err("EmptyItemProducer returns an Err");

        assert_eq!(ErrorKind::NoValue, res.kind());
    }
}
