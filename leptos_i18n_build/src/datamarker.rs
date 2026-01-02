use icu_provider::{DataMarker, DataMarkerInfo};
use leptos_i18n_parser::formatters;
use leptos_i18n_parser::parse_locales::locale::{
    BuildersKeysInner, InterpolOrLit, LocaleValue, RangeOrPlural,
};
use std::collections::HashSet;

/// This enum represent the different `Formatters` and options your translations could be using.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FormatterOptions {
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

pub fn find_used_datamarker(
    markers: &BuildersKeysInner,
    used_icu_markers: &mut HashSet<FormatterOptions>,
) {
    for locale_value in markers.0.values() {
        match locale_value {
            LocaleValue::Subkeys { keys, .. } => find_used_datamarker(keys, used_icu_markers),
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
                        used_icu_markers.insert(FormatterOptions::Plurals);
                    }

                    for formatter in &var_infos.formatters {
                        let dk = if formatter.is::<formatters::currency::CurrencyFormatter>() {
                            FormatterOptions::FormatCurrency
                        } else if formatter.is::<formatters::nums::NumberFormatter>() {
                            FormatterOptions::FormatNums
                        } else if formatter.is::<formatters::datetime::DateFormatter>()
                            || formatter.is::<formatters::datetime::DateTimeFormatter>()
                            || formatter.is::<formatters::datetime::TimeFormatter>()
                        {
                            FormatterOptions::FormatDateTime
                        } else if formatter.is::<formatters::list::ListFormatter>() {
                            FormatterOptions::FormatList
                        } else {
                            continue;
                        };
                        used_icu_markers.insert(dk);
                    }
                }
            }
        }
    }
}

pub fn get_markers(
    used_icu_markers: impl IntoIterator<Item = FormatterOptions>,
) -> impl Iterator<Item = DataMarkerInfo> {
    used_icu_markers
        .into_iter()
        .flat_map(FormatterOptions::into_data_markers)
}

impl FormatterOptions {
    /// Return a `Vec<DataMarkerInfo>` needed to use the given option.
    pub fn into_data_markers(self) -> Vec<DataMarkerInfo> {
        match self {
            FormatterOptions::Plurals => icu::calendar::provider::MARKERS.to_vec(),
            FormatterOptions::FormatDateTime => [
                icu::datetime::provider::MARKERS,
                icu::plurals::provider::MARKERS,
                icu::decimal::provider::MARKERS,
                icu::calendar::provider::MARKERS,
            ]
            .iter()
            .flat_map(|m| m.to_vec())
            .collect(),
            FormatterOptions::FormatList => icu::list::provider::MARKERS.to_vec(),
            FormatterOptions::FormatNums => icu::decimal::provider::MARKERS.to_vec(),
            FormatterOptions::FormatCurrency => [
                icu::decimal::provider::MARKERS,
                &[icu::experimental::dimension::provider::currency::essentials::CurrencyEssentialsV1::INFO],
            ]
            .iter()
            .flat_map(|m| m.to_vec())
            .collect(),
        }
    }
}
