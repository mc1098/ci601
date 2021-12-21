mod service;

use biblatex::{Bibliography, Entry};
use eyre::eyre;
use service::{BibTexService, DoiService};

#[inline]
fn unique_entry_check<P>(bibliography: Bibliography, predicate: P) -> eyre::Result<()>
where
    P: Fn(&Entry) -> eyre::Result<()>,
{
    bibliography.iter().try_fold((), |_, e| predicate(e))
}

pub fn get_by_doi(doi: &str) -> Result<String, eyre::Report> {
    DoiService::new(doi)
        .get_bibtex()
        .map(|entry| entry.to_bibtex_string())
}

fn add_by_service(service: impl BibTexService, bib: &mut impl std::io::Write) -> eyre::Result<()> {
    let entry = service.get_bibtex()?;
    bib.write_all(entry.to_bibtex_string().as_bytes())?;
    Ok(())
}

pub fn add_by_doi(
    doi: &str,
    bib: &mut impl std::io::Write,
    bibliography: Bibliography,
) -> Result<(), eyre::Report> {
    // check if the current bibliography contains the entry already before doing the http request.
    unique_entry_check(bibliography, |e| {
        e.doi()
            .and_then(|d| (d != doi).then(|| ()))
            .ok_or_else(|| eyre!("A bibtex entry already exists with the doi of '{}'"))
    })?;

    let service = DoiService::new(doi);
    add_by_service(service, bib)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BIBTEX_ENTRY_1: &str = include_str!("../tests/data/bibtex1.bib");

    #[test]
    #[should_panic(expected = "Duplicate found")]
    fn err_on_duplicate_entry() {
        let bib = Bibliography::parse(BIBTEX_ENTRY_1).unwrap();
        unique_entry_check(bib, |e| {
            if e.key == "Edelkamp_2019" {
                Err(eyre!("Duplicate found"))
            } else {
                Ok(())
            }
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "already exists with the doi")]
    fn duplicate_add_on_doi_errors() {
        let bib = Bibliography::parse(BIBTEX_ENTRY_1).unwrap();
        add_by_doi("10.1007/s00453-019-00634-0", &mut Vec::new(), bib).unwrap();
    }
}
