use std::collections::HashSet;

use icu_datagen::prelude::DataKey;
use leptos_i18n_parser::{
    parse_locales::locale::{BuildersKeysInner, InterpolOrLit, LocaleValue, RangeOrPlural},
    utils::formatter::Formatter,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DataK {
    Plurals,
    FormatDateTime,
    FormatList,
    FormatNums,
}

pub fn find_used_datakey(keys: &BuildersKeysInner, used_icu_keys: &mut HashSet<DataK>) {
    for locale_value in keys.0.values() {
        match locale_value {
            LocaleValue::Subkeys { keys, .. } => find_used_datakey(keys, used_icu_keys),
            LocaleValue::Value(InterpolOrLit::Lit(_)) => {} // skip literals
            LocaleValue::Value(InterpolOrLit::Interpol(interpolation_keys)) => {
                for (_, var_infos) in interpolation_keys.iter_vars() {
                    if matches!(var_infos.range_count, Some(RangeOrPlural::Plural)) {
                        used_icu_keys.insert(DataK::Plurals);
                    }

                    for formatter in &var_infos.formatters {
                        let dk = match formatter {
                            Formatter::None => continue,
                            Formatter::Number => DataK::FormatNums,
                            Formatter::Date(_) | Formatter::Time(_) | Formatter::DateTime(_, _) => {
                                DataK::FormatDateTime
                            }
                            Formatter::List(_, _) => DataK::FormatList,
                        };
                        used_icu_keys.insert(dk);
                    }
                }
            }
        }
    }
}

pub fn get_keys(used_icu_keys: HashSet<DataK>) -> HashSet<DataKey> {
    used_icu_keys
        .into_iter()
        .flat_map(DataK::into_data_keys)
        .collect()
}

impl DataK {
    pub fn into_data_keys(self) -> Vec<DataKey> {
        match self {
            DataK::Plurals => icu_datagen::keys(&["plurals/cardinal@1", "plurals/ordinal@1"]),
            DataK::FormatDateTime => icu_datagen::keys(&[
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
            DataK::FormatList => icu_datagen::keys(&["list/and@1", "list/or@1", "list/unit@1"]),
            DataK::FormatNums => icu_datagen::keys(&["decimal/symbols@1"]),
        }
    }
}
