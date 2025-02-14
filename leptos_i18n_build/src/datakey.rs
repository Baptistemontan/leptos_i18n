use std::collections::HashSet;

use icu_datagen::prelude::DataKey;
use leptos_i18n_parser::{
    parse_locales::locale::{BuildersKeysInner, InterpolOrLit, LocaleValue, RangeOrPlural},
    utils::formatter::Formatter,
};

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

pub fn find_used_datakey(keys: &BuildersKeysInner, used_icu_keys: &mut HashSet<Options>) {
    for locale_value in keys.0.values() {
        match locale_value {
            LocaleValue::Subkeys { keys, .. } => find_used_datakey(keys, used_icu_keys),
            LocaleValue::Value(InterpolOrLit::Lit(_)) => {} // skip literals
            LocaleValue::Value(InterpolOrLit::Interpol(interpolation_keys)) => {
                for (_, var_infos) in interpolation_keys.iter_vars() {
                    if matches!(var_infos.range_count, Some(RangeOrPlural::Plural)) {
                        used_icu_keys.insert(Options::Plurals);
                    }

                    for formatter in &var_infos.formatters {
                        let dk = match formatter {
                            Formatter::None => continue,
                            Formatter::Number(_) => Options::FormatNums,
                            Formatter::Date(_) | Formatter::Time(_) | Formatter::DateTime(_, _) => {
                                Options::FormatDateTime
                            }
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

pub fn get_keys(used_icu_keys: impl IntoIterator<Item = Options>) -> impl Iterator<Item = DataKey> {
    used_icu_keys.into_iter().flat_map(Options::into_data_keys)
}

impl Options {
    /// Return a `Vec<DataKey>` needed to use the given option.
    pub fn into_data_keys(self) -> Vec<DataKey> {
        match self {
            Options::Plurals => icu_datagen::keys(&["plurals/cardinal@1", "plurals/ordinal@1"]),
            Options::FormatDateTime => icu_datagen::keys(&[
                "datetime/timesymbols@1",
                "datetime/timelengths@1",
                "datetime/skeletons@1",
                "plurals/ordinal@1",
                "datetime/week_data@1",
                "decimal/symbols@1",
                "datetime/gregory/datelengths@1",
                "datetime/gregory/datesymbols@1",
                "datetime/buddhist/datelengths@1",
                "datetime/buddhist/datesymbols@1",
                "calendar/chinesecache@1",
                "datetime/chinese/datelengths@1",
                "datetime/chinese/datesymbols@1",
                "datetime/coptic/datelengths@1",
                "datetime/coptic/datesymbols@1",
                "calendar/dangicache@1",
                "datetime/dangi/datelengths@1",
                "datetime/dangi/datesymbols@1",
                "datetime/ethiopic/datelengths@1",
                "datetime/ethiopic/datesymbols@1",
                "datetime/hebrew/datelengths@1",
                "datetime/hebrew/datesymbols@1",
                "datetime/indian/datelengths@1",
                "datetime/indian/datesymbols@1",
                "datetime/islamic/datelengths@1",
                "datetime/islamic/datesymbols@1",
                "calendar/islamicobservationalcache@1",
                "calendar/islamicummalquracache@1",
                "datetime/japanese/datelengths@1",
                "datetime/japanese/datesymbols@1",
                "calendar/japanese@1",
                "datetime/japanext/datelengths@1",
                "datetime/japanext/datesymbols@1",
                "calendar/japanext@1",
                "datetime/persian/datelengths@1",
                "datetime/persian/datesymbols@1",
                "datetime/roc/datelengths@1",
                "datetime/roc/datesymbols@1",
            ]),
            Options::FormatList => icu_datagen::keys(&["list/and@1", "list/or@1", "list/unit@1"]),
            Options::FormatNums => icu_datagen::keys(&["decimal/symbols@1"]),
            Options::FormatCurrency => icu_datagen::keys(&[
                "decimal/digits@1",
                "decimal/symbols@2",
                "currency/essentials@1",
            ]),
        }
    }
}
