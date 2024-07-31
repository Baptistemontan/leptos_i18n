//! A lot of the code in this module is shamefully taken from `fluent-template`, I would have used there crate directly if those implementations where public.
//! see <https://github.com/XAMPPRocky/fluent-templates>
//!
//! I then specialized it for the use case of this crate.
use unic_langid::LanguageIdentifier;

use crate::Locale;

pub fn filter_matches<BL: Locale, L: Locale<BL>>(
    requested: &[LanguageIdentifier],
    available: &[L],
) -> Vec<L> {
    let mut supported_locales = vec![];

    let mut available_locales: Vec<L> = available.to_vec();

    for req in requested.iter().cloned() {
        macro_rules! test_strategy {
            ($self_as_range:expr) => {{
                let mut match_found = false;
                available_locales.retain(|locale| {
                    if locale.as_ref().matches(&req, $self_as_range, false) {
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
        if req.language.is_empty() {
            continue;
        }
    }

    supported_locales.sort_by(|x, y| {
        let x_specificity = into_specificity(x.as_ref());
        let y_specificity = into_specificity(y.as_ref());
        x_specificity.cmp(&y_specificity).reverse()
    });

    supported_locales
}

fn into_specificity(lang: &LanguageIdentifier) -> usize {
    // let parts = lang.into_parts();
    let mut specificity = 0;
    // Script
    if lang.script.is_some() {
        specificity += 1;
    }
    // Region
    if lang.region.is_some() {
        specificity += 1;
    }

    // variant
    specificity += lang.variants().len();

    specificity
}

pub fn find_match<BL: Locale, L: Locale<BL>>(
    requested: &[LanguageIdentifier],
    available: &[L],
) -> L {
    filter_matches(requested, available)
        .first()
        .copied()
        .unwrap_or_default()
}

/// This function is taken from `fluent-langneg`.
/// see <https://github.com/projectfluent/fluent-langneg-rs>
///
/// Yes I could have imported the crate as this is public, but well we are already far into the strealing process anyway
pub fn convert_vec_str_to_langids_lossy<'a, I, J>(input: I) -> Vec<LanguageIdentifier>
where
    I: IntoIterator<Item = J>,
    J: AsRef<[u8]> + 'a,
{
    input
        .into_iter()
        .filter_map(|t| LanguageIdentifier::from_bytes(t.as_ref()).ok())
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
    use unic_langid::langid;

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
    }
}
