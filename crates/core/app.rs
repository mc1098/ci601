use eyre::{eyre, Result};
use seb::ast::{Biblio, Entry};

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
    entries.iter().map(|e| e.title().to_string()).collect()
}

pub fn check_entry_field_duplication(bib: &Biblio, name: &str, value: &str) -> eyre::Result<()> {
    if bib.contains_field(name, |f| &**f == value) {
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
    use seb::ast::{EntryData, Manual, QuotedString};
    use std::collections::HashMap;

    let mut bib = Biblio::new(vec![]);
    let name = "doi";
    let doi = QuotedString::new("test".to_owned());

    assert!(check_entry_field_duplication(&bib, name, &doi).is_ok());

    let data = Manual {
        title: QuotedString::new("test".to_owned()),
        optional: HashMap::from([(name.to_owned(), doi.clone())]),
    };

    bib.insert(Entry {
        citation_key: String::new(),
        entry_data: EntryData::Manual(data),
    });

    assert!(check_entry_field_duplication(&bib, name, &doi).is_err());
}
