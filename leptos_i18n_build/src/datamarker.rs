use icu_provider::{DataMarker, DataMarkerInfo};
use leptos_i18n_parser::{
    formatters::{self, FormatterToTokens, VarBounds},
    parse_locales::locale::{BuildersKeysInner, InterpolOrLit, LocaleValue, RangeOrPlural},
};
use std::{
    any::{Any, TypeId},
    collections::HashSet,
};

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

fn check_bound<T: Any>(bound: &dyn FormatterToTokens) -> bool {
    let ty_id = TypeId::of::<T>();
    bound.is(ty_id)
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

                    for bound in &var_infos.bounds {
                        let dk = match bound {
                            VarBounds::Formatted { to_tokens, .. } => {
                                if check_bound::<formatters::currency::CurrencyFormatter>(
                                    &**to_tokens,
                                ) {
                                    FormatterOptions::FormatCurrency
                                } else if check_bound::<formatters::nums::NumberFormatter>(
                                    &**to_tokens,
                                ) {
                                    FormatterOptions::FormatNums
                                } else if check_bound::<formatters::datetime::DateFormatter>(
                                    &**to_tokens,
                                ) || check_bound::<formatters::datetime::DateTimeFormatter>(
                                    &**to_tokens,
                                ) || check_bound::<formatters::datetime::TimeFormatter>(
                                    &**to_tokens,
                                ) {
                                    FormatterOptions::FormatDateTime
                                } else if check_bound::<formatters::list::ListFormatter>(
                                    &**to_tokens,
                                ) {
                                    FormatterOptions::FormatList
                                } else {
                                    continue;
                                }
                            }
                            _ => continue,
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
            FormatterOptions::Plurals => icu::plurals::provider::MARKERS.to_vec(),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every option must at least bake the data of the ICU component it stands for,
    /// otherwise the generated provider compiles but fails at runtime with a
    /// missing data error.
    #[test]
    fn options_bake_the_data_of_their_own_component() {
        let contains = |option: FormatterOptions, markers: &[DataMarkerInfo]| {
            let baked = option.into_data_markers();
            markers.iter().all(|marker| baked.contains(marker))
        };

        assert!(contains(
            FormatterOptions::Plurals,
            icu::plurals::provider::MARKERS
        ));
        assert!(contains(
            FormatterOptions::FormatDateTime,
            icu::datetime::provider::MARKERS
        ));
        assert!(contains(
            FormatterOptions::FormatList,
            icu::list::provider::MARKERS
        ));
        assert!(contains(
            FormatterOptions::FormatNums,
            icu::decimal::provider::MARKERS
        ));
        assert!(contains(
            FormatterOptions::FormatCurrency,
            icu::decimal::provider::MARKERS
        ));
    }

    /// `Plurals` used to be mapped to the calendar markers, which baked Japanese
    /// calendar and week data while leaving the plural rules out entirely.
    #[test]
    fn plurals_does_not_bake_calendar_data() {
        let baked = FormatterOptions::Plurals.into_data_markers();
        for marker in icu::calendar::provider::MARKERS {
            assert!(
                !baked.contains(marker),
                "`FormatterOptions::Plurals` should not pull in calendar data"
            );
        }
    }
}
