use std::collections::HashMap;

use super::parsed_value::ParsedValue;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum PluralForm {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl PluralForm {
    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "zero" => Some(PluralForm::Zero),
            "one" => Some(PluralForm::One),
            "two" => Some(PluralForm::Two),
            "few" => Some(PluralForm::Few),
            "many" => Some(PluralForm::Many),
            "other" => Some(PluralForm::Other),
            _ => None,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            PluralForm::Zero => "zero",
            PluralForm::One => "one",
            PluralForm::Two => "two",
            PluralForm::Few => "few",
            PluralForm::Many => "many",
            PluralForm::Other => "other",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plurals {
    pub other: Box<ParsedValue>,
    pub forms: HashMap<PluralForm, ParsedValue>,
}
