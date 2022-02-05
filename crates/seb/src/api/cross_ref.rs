use serde::Deserialize;

use crate::{
    api::format_api,
    ast::{Biblio, BiblioResolver},
    format::BibTex,
    Error, ErrorKind,
};

use super::Client;

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
) -> Result<Result<Biblio, BiblioResolver>, Error> {
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
    if items.is_empty() {
        Err(Error::new(
            ErrorKind::NoValue,
            "No entries found for that title",
        ))
    } else {
        Ok(items.into_iter().map(EntryStub::into_tuple).collect())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        api::{impl_text_producer, MockJsonClient},
        ErrorKind,
    };

    use super::QueryResult;

    const ENTRY_STUB_JSON: &str = include_str!("../../../../tests/data/crossref_entry_stub.json");

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
        let res = super::get_entry_stubs_by_title::<MockJsonClient<ValidJsonProducer>>("test")
            .expect("ValidJsonProducer always produces a valid json String to be deserialized");

        assert_eq!(20, res.len());
    }

    #[test]
    fn empty_item_returns_no_value_error() {
        let res = super::get_entry_stubs_by_title::<MockJsonClient<EmptyItemProducer>>("test")
            .expect_err("EmptyItemProducer returns an Err");

        assert_eq!(ErrorKind::NoValue, res.kind());
    }
}
