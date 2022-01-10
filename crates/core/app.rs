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
    entries.iter().map(|e| e.title().to_owned()).collect()
}

pub fn check_entry_field_duplication(bib: &Biblio, name: &str, value: &str) -> eyre::Result<()> {
    if bib.contains_field(name, |f| f.value() == value) {
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
    use seb::ast::QuotedString;

    let mut bib = Biblio::new(vec![]);
    let name = "doi";
    let doi = QuotedString::new("test".to_owned());

    assert!(check_entry_field_duplication(&bib, name, &doi).is_ok());

    bib.insert(Entry {
        cite: String::new(),
        title: QuotedString::new("test".to_owned()),
        variant: seb::ast::EntryType::Book,
        fields: vec![seb::ast::Field {
            name: name.to_owned(),
            value: doi.clone(),
        }],
    });

    assert!(check_entry_field_duplication(&bib, name, &doi).is_err());
}
