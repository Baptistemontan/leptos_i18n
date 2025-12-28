//! A lot of the code in this module is shamefully taken from `fluent-template`, I would have used their crate directly if those implementations where public.
//! see <https://github.com/XAMPPRocky/fluent-templates>
//!
//! I then specialized it for the use case of this crate.
use icu_locale::{
    LanguageIdentifier,
    subtags::{Language, Variant},
};

use crate::Locale;

fn lang_matches(lhs: &Language, rhs: &Language, self_as_range: bool, other_as_range: bool) -> bool {
    (self_as_range && lhs.is_unknown()) || (other_as_range && rhs.is_unknown()) || lhs == rhs
}

fn subtag_matches<P: PartialEq>(
    subtag1: &Option<P>,
    subtag2: &Option<P>,
    as_range1: bool,
    as_range2: bool,
) -> bool {
    (as_range1 && subtag1.is_none()) || (as_range2 && subtag2.is_none()) || subtag1 == subtag2
}

fn subtags_match(
    subtag1: &[Variant],
    subtag2: &[Variant],
    as_range1: bool,
    as_range2: bool,
) -> bool {
    (as_range1 && subtag1.is_empty()) || (as_range2 && subtag2.is_empty()) || subtag1 == subtag2
}

fn lang_id_matches(
    lhs: &LanguageIdentifier,
    rhs: &LanguageIdentifier,
    self_as_range: bool,
    other_as_range: bool,
) -> bool {
    lang_matches(&lhs.language, &rhs.language, self_as_range, other_as_range)
        && subtag_matches(&lhs.script, &rhs.script, self_as_range, other_as_range)
        && subtag_matches(&lhs.region, &rhs.region, self_as_range, other_as_range)
        && subtags_match(&lhs.variants, &rhs.variants, self_as_range, other_as_range)
}

pub fn filter_matches<L: Locale>(requested: &[LanguageIdentifier], available: &[L]) -> Vec<L> {
    let mut supported_locales = vec![];

    let mut available_locales: Vec<L> = available.to_vec();

    for req in requested.iter().cloned() {
        macro_rules! test_strategy {
            ($self_as_range:expr) => {{
                let mut match_found = false;
                available_locales.retain(|locale| {
                    if lang_id_matches(locale.as_langid(), &req, $self_as_range, false) {
                        match_found = true;
                        supported_locales.push(*locale);
                        return false;
                    }
                    true
                });
            }};
        }

        // 1) Try to find a simple (case-insensitive) string match for the request.
        test_strategy!(false);

        // 2) Try to match against the available locales treated as ranges.
        test_strategy!(true);

        // Per Unicode TR35, 4.4 Locale Matching, we don't add likely subtags to
        // requested locales, so we'll skip it from the rest of the steps.
        if req.language.is_unknown() {
            continue;
        }
    }

    supported_locales
}

pub fn find_match<L: Locale>(requested: &[LanguageIdentifier], available: &[L]) -> L {
    filter_matches(requested, available)
        .first()
        .copied()
        .unwrap_or_default()
}

/// This function is taken from `fluent-langneg`.
/// see <https://github.com/projectfluent/fluent-langneg-rs>
///
/// Yes I could have imported the crate as this is public, but well we are already far into the stealing process anyway
pub fn convert_vec_str_to_langids_lossy<'a, I, J>(input: I) -> Vec<LanguageIdentifier>
where
    I: IntoIterator<Item = J>,
    J: AsRef<[u8]> + 'a,
{
    input
        .into_iter()
        .filter_map(|t| LanguageIdentifier::try_from_locale_bytes(t.as_ref()).ok())
        .collect()
}

#[cfg(test)]
mod test {
    leptos_i18n_macro::declare_locales! {
        path: crate,
        default: "de",
        locales: ["de", "en-US", "de-DE", "de-CH", "en", "fr", "fr-FR"],
        de: {},
        en_US: {},
        de_DE: {},
        de_CH: {},
        en: {},
        fr: {},
        fr_FR: {}
    }

    use super::{filter_matches, find_match};
    use i18n::Locale;

    use icu_locale::langid;

    #[test]
    fn test_hirarchy() {
        const LOCALES: &[Locale] = &[Locale::de, Locale::en_US, Locale::de_DE, Locale::de_CH];

        let res = filter_matches(&[langid!("de")], LOCALES);
        assert_eq!(res, [Locale::de]);

        let res = filter_matches(&[langid!("de-DE")], LOCALES);
        assert_eq!(res, [Locale::de_DE, Locale::de]);

        let res = filter_matches(&[langid!("de-CH")], LOCALES);
        assert_eq!(res, [Locale::de_CH, Locale::de]);
    }

    #[test]
    fn test_find_match() {
        let res = find_match(
            &[langid!("de-DE")],
            &[Locale::de_DE, Locale::de, Locale::en_US, Locale::de_CH],
        );
        assert_eq!(res, Locale::de_DE);

        let res = find_match(&[langid!("de-DE")], &[Locale::de, Locale::de_DE]);
        assert_eq!(res, Locale::de_DE);

        let res = find_match(
            &[langid!("en"), langid!("de-DE")],
            &[Locale::en, Locale::de_DE],
        );
        assert_eq!(res, Locale::en);
    }
}
