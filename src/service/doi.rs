use super::BibTexService;
use biblatex::{Bibliography, Entry};
use eyre::eyre;
use log::*;

pub struct DoiService<'doi> {
    doi: &'doi str,
}

impl<'doi> DoiService<'doi> {
    pub fn new(doi: &'doi str) -> Self {
        Self { doi }
    }
}

impl BibTexService for DoiService<'_> {
    fn get_bibtex(&self) -> Result<Entry, eyre::Report> {
        let doi = self.doi;
        info!("searching for doi '{}' at api.crossref.org", doi);
        let url = format!(
            "https://api.crossref.org/works/{}/transform/application/x-bibtex",
            doi
        );
        let client = reqwest::blocking::Client::new();
        let resp = client.get(url).send().and_then(|r| r.text());

        match resp {
            Ok(text) => {
                if let Some(entry) =
                    Bibliography::parse(&text).and_then(|biblio| biblio.into_iter().next())
                {
                    trace!("Found reference and successfully parsed into a bibtex entry");
                    Ok(entry)
                } else {
                    Err(eyre!(
                        "doi reference found but information could not used to build a bibtex entry"
                    ))
                }
            }
            Err(err) => {
                let ret = match err.status() {
                    Some(reqwest::StatusCode::NOT_FOUND) => {
                        eyre!("No reference found with the doi of '{}'", doi)
                    }
                    _ => {
                        eyre!("Oops - there was a problem trying to find the reference by doi")
                    }
                };

                Err(ret)
            }
        }
    }
}
