use crate::{
    parser::{options::LocaleName, raw_value::component::Component},
    utils::{Key, KeyPath, Loc},
};

#[derive(Debug, Clone, PartialEq)]
pub enum DummyArg {
    Variable(Key),
    Component(Key),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dummy {
    dummies: Vec<DummyArg>,
}

impl Dummy {
    pub fn parse(s: &str, locale: &LocaleName) -> Self {
        let mut dummies = Vec::new();
        Self::parse_inner(s, &mut dummies, locale);
        Dummy { dummies }
    }

    fn parse_inner(s: &str, dummies: &mut Vec<DummyArg>, locale: &LocaleName) {
        if Self::find_component(s, dummies, locale).is_none() {
            Self::find_var(s, dummies);
        }
    }

    fn find_component(s: &str, dummies: &mut Vec<DummyArg>, locale: &LocaleName) -> Option<()> {
        let dummy_key_path = KeyPath::default();
        let dummy_loc = Loc {
            key_path: &dummy_key_path,
            locale,
        };

        let component = Component::find_component(s, dummy_loc)?.ok()?;

        dummies.push(DummyArg::Component(component.key));

        Self::parse_inner(component.before, dummies, locale);

        if let Some(inner) = component.inner {
            Self::parse_inner(inner, dummies, locale);
        }
        Self::parse_inner(component.after, dummies, locale);

        Self::find_var(component.attrs, dummies)
    }

    fn find_var(value: &str, dummies: &mut Vec<DummyArg>) -> Option<()> {
        let (before, rest) = value.split_once("{{")?;
        let (ident, after) = rest.split_once("}}")?;

        let ident = if let Some((ident, _)) = ident.split_once(',') {
            ident.trim()
        } else {
            ident.trim()
        };

        let key = Key::new(&format!("var_{ident}"))?;

        dummies.push(DummyArg::Variable(key));

        Self::find_var(before, dummies);
        Self::find_var(after, dummies);

        Some(())
    }
}
