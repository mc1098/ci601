use eyre::{eyre, Result};
use seb::Entry;

fn user_select(mut entries: Vec<Entry>) -> Result<Entry> {
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Confirm entry")
        .default(0)
        .items(&entries)
        .interact_opt()
        .unwrap();

    if let Some(index) = selection {
        Ok(entries.remove(index))
    } else {
        Err(eyre!("No entry confirmed - cancelling operation"))
    }
}

pub fn select_entry_by_doi(doi: &str) -> Result<Entry> {
    let entries = seb::entries_by_doi(doi)?;

    user_select(entries)
}

pub fn select_entry_by_isbn(isbn: &str) -> Result<Entry> {
    let entries = seb::entries_by_isbn(isbn)?;

    user_select(entries)
}

pub fn check_entry_field_duplication(
    bib: &seb::Biblio,
    name: &str,
    value: &str,
) -> eyre::Result<()> {
    if bib.contains_field(name, |f| f.value == value) {
        Err(eyre!(
            "An entry already exists with a {} field with the value of '{}'.",
            name,
            value
        ))
    } else {
        Ok(())
    }
}

#[test]
fn field_dup_macro() {
    let mut bib = seb::Biblio::new(vec![]);
    let name = "doi";
    let doi = "test";

    assert!(check_entry_field_duplication(&bib, name, doi).is_ok());

    bib.insert(seb::Entry {
        cite: String::new(),
        variant: seb::EntryType::Book,
        fields: vec![seb::Field {
            name: name.to_owned(),
            value: doi.to_owned(),
        }],
    });

    assert!(check_entry_field_duplication(&bib, name, doi).is_err());
}
