use eyre::{eyre, Result};
use seb::Entry;

pub fn user_select(mut entries: Vec<Entry>) -> Result<Entry> {
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Confirm entry")
        .default(0)
        .items(&entries_titles(&entries))
        .interact_opt()
        .unwrap();

    if let Some(index) = selection {
        Ok(entries.remove(index))
    } else {
        Err(eyre!("No entry confirmed - cancelling operation"))
    }
}

fn entries_titles(entries: &[Entry]) -> Vec<String> {
    entries.iter().map(|e| e.title.clone()).collect()
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
        title: "test".to_owned(),
        variant: seb::EntryType::Book,
        fields: vec![seb::Field {
            name: name.to_owned(),
            value: doi.to_owned(),
        }],
    });

    assert!(check_entry_field_duplication(&bib, name, doi).is_err());
}
