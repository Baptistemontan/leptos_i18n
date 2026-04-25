use leptos_i18n_parser::extraction::{Keys, ValuesOrSubkeys};

pub fn gen_keys_doc(docs: &mut String, keys: &Keys) -> core::fmt::Result {
    use core::fmt::Write;
    let mut keys_iter = keys
        .values
        .iter()
        .filter_map(|(key, value)| match value {
            ValuesOrSubkeys::Values { .. } => Some(key),
            ValuesOrSubkeys::Subkeys { .. } => None,
        })
        .peekable();

    if keys_iter.peek().is_some() {
        writeln!(docs, "\n## Keys :")?;
        for key in keys_iter {
            writeln!(docs, "- `{}`", key)?;
        }
    }

    let mut keys_iter = keys
        .values
        .iter()
        .filter_map(|(key, value)| match value {
            ValuesOrSubkeys::Values { .. } => None,
            ValuesOrSubkeys::Subkeys { .. } => Some(key),
        })
        .peekable();

    if keys_iter.peek().is_some() {
        writeln!(docs, "## Subkeys :")?;
        for key in keys_iter {
            writeln!(docs, "- `{}`", key)?;
        }
    }

    Ok(())
}
