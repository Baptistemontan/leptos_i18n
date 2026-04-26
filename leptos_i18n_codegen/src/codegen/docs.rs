use leptos_i18n_parser::extraction::{Keys, ValuesOrSubkeys};

use crate::codegen::builders::infos::{Field, VarOrComp};

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

fn gen_fields_docs(docs: &mut String, fields: &[Field]) -> core::fmt::Result {
    use core::fmt::Write;
    let mut variables = fields
        .iter()
        .filter_map(|field| match &field.var_or_comp {
            VarOrComp::Var { bounds, plural } => {
                let key = field.key.name.strip_prefix("var_")?;
                Some((key, bounds.as_slice(), *plural))
            }
            VarOrComp::Comp { .. } => None,
        })
        .peekable();

    if variables.peek().is_some() {
        writeln!(docs, "## Vars :")?;
        for (key, bounds, plural) in variables {
            let _ = bounds;
            match plural {
                true => writeln!(docs, "- `{}` (plural count)", key)?,
                false => writeln!(docs, "- `{}`", key)?,
            }
        }
    }

    let mut components = fields
        .iter()
        .filter_map(|field| match &field.var_or_comp {
            VarOrComp::Var { .. } => None,
            VarOrComp::Comp { self_closed, .. } => {
                let key = field.key.name.strip_prefix("comp_")?;
                Some((key, *self_closed))
            }
        })
        .peekable();

    if components.peek().is_some() {
        writeln!(docs, "## Components :")?;
        for (key, self_closed) in components {
            if self_closed {
                writeln!(docs, "- `<{}/>`", key)?;
            } else {
                writeln!(docs, "-  `<{}>`", key)?;
            }
        }
    }

    Ok(())
}
