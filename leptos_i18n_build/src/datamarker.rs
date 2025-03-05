use icu_provider::{DataMarker, DataMarkerInfo};
use leptos_i18n_parser::{
    parse_locales::locale::{BuildersKeysInner, InterpolOrLit, LocaleValue, RangeOrPlural},
    utils::formatter::Formatter,
};
use std::collections::HashSet;

/// This enum represent the different `Fromatters` and options your translations could be using.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Options {
    /// Use of plurals.
    Plurals,
    /// Use of the `date`, `time` or `datetime` formatter.
    FormatDateTime,
    /// Use of the `list` formatter.
    FormatList,
    /// Use of the `number` formatter.
    FormatNums,
    /// Use of the `currency` formatter.
    FormatCurrency,
}

pub fn find_used_datamarker(keys: &BuildersKeysInner, used_icu_keys: &mut HashSet<Options>) {
    for locale_value in keys.0.values() {
        match locale_value {
            LocaleValue::Subkeys { keys, .. } => find_used_datamarker(keys, used_icu_keys),
            LocaleValue::Value {
                // skip literals
                value: InterpolOrLit::Lit(_),
                ..
            } => {}
            LocaleValue::Value {
                value: InterpolOrLit::Interpol(interpolation_keys),
                ..
            } => {
                for (_, var_infos) in interpolation_keys.iter_vars() {
                    if matches!(var_infos.range_count, Some(RangeOrPlural::Plural)) {
                        used_icu_keys.insert(Options::Plurals);
                    }

                    for formatter in &var_infos.formatters {
                        let dk = match formatter {
                            Formatter::None => continue,
                            Formatter::Number(_) => Options::FormatNums,
                            Formatter::Date(_, _)
                            | Formatter::Time(_, _, _)
                            | Formatter::DateTime(_, _, _) => Options::FormatDateTime,
                            Formatter::List(_, _) => Options::FormatList,
                            Formatter::Currency(_, _) => Options::FormatCurrency,
                        };
                        used_icu_keys.insert(dk);
                    }
                }
            }
        }
    }
}

pub fn get_keys(
    used_icu_keys: impl IntoIterator<Item = Options>,
) -> impl Iterator<Item = DataMarkerInfo> {
    used_icu_keys.into_iter().flat_map(Options::into_data_keys)
}

impl Options {
    /// Return a `Vec<DataMarkerInfo>` needed to use the given option.
    pub fn into_data_keys(self) -> Vec<DataMarkerInfo> {
        match self {
            Options::Plurals => icu::calendar::provider::MARKERS.to_vec(),
            Options::FormatDateTime => [
                icu::datetime::provider::MARKERS,
                icu::plurals::provider::MARKERS,
                icu::decimal::provider::MARKERS,
                icu::calendar::provider::MARKERS,
            ]
            .iter()
            .flat_map(|m| m.to_vec())
            .collect(),
            Options::FormatList => icu::list::provider::MARKERS.to_vec(),
            Options::FormatNums => icu::decimal::provider::MARKERS.to_vec(),
            Options::FormatCurrency => [
                icu::decimal::provider::MARKERS,
                &[icu::experimental::dimension::provider::currency::CurrencyEssentialsV1::INFO],
            ]
            .iter()
            .flat_map(|m| m.to_vec())
            .collect(),
        }
    }
}
