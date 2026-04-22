use std::{collections::BTreeMap, path::PathBuf};

use serde::de::{DeserializeSeed, MapAccess, value::MapAccessDeserializer};

use crate::{
    error::{Diagnostics, Error, Result},
    formatters::Formatters,
    parser::locale::{RawValueOrSubkeys, RawValues},
    utils::{Key, KeyPath, Loc, Location},
};

pub mod dummy;
pub mod locale;
pub mod options;
pub mod raw_value;

use dummy::Dummy;
use locale::RawLocalesOrNamespaces;
use options::Config;
use raw_value::{RawLiteral, RawValue};

pub const DEFAULT_LOCALES_PATH: &str = "locales";

#[derive(Debug)]
pub struct RawParsedLocales {
    pub values: RawLocalesOrNamespaces,
    pub cfg: Config,
    pub diag: Diagnostics,
    pub tracked_files: Vec<String>,
}

pub fn parse_locales_raw(
    cargo_manifest_dir: Option<PathBuf>,
    cfg: Config,
) -> Result<RawParsedLocales> {
    let mut cargo_manifest_dir = unwrap_manifest_dir(cargo_manifest_dir)?;

    let diag = Diagnostics::new();

    // let cfg_file = ConfigFile::new(&mut cargo_manifest_dir)?;

    let mut tracked_files = Vec::with_capacity(cfg.locales.len() * cfg.namespaces.len().max(1));

    let values =
        RawLocalesOrNamespaces::new(&mut cargo_manifest_dir, &diag, &mut tracked_files, &cfg)?;

    let raw_parsed_locales = RawParsedLocales {
        values,
        cfg,
        diag,
        tracked_files,
    };

    Ok(raw_parsed_locales)
}

fn get_manifest_dir() -> Result<PathBuf> {
    let path = std::env::var("CARGO_MANIFEST_DIR")
        .map(Into::into)
        .map_err(Error::CargoDirEnvNotPresent)?;

    Ok(path)
}

fn unwrap_manifest_dir(cargo_manifest_dir: Option<PathBuf>) -> Result<PathBuf> {
    match cargo_manifest_dir {
        Some(path) => Ok(path),
        None => get_manifest_dir(),
    }
}

pub type ParseFn = fn(&ParseContext, &str) -> Option<Result<RawValue, ()>>;

#[derive(Clone, Copy)]
pub struct ParseContext<'a> {
    pub loc: Loc<'a>,
    pub formatters: &'a Formatters,
    pub diag: &'a Diagnostics,
    pub parse_fns: &'a [ParseFn],
}

impl From<&'_ ParseContext<'_>> for Location {
    fn from(ctx: &'_ ParseContext) -> Self {
        ctx.loc.into()
    }
}

impl From<ParseContext<'_>> for Location {
    fn from(ctx: ParseContext<'_>) -> Self {
        ctx.loc.into()
    }
}

#[derive(Clone)]
pub struct ValuesSeed<'a> {
    pub name: Key,
    pub top_locale_name: Key,
    pub key_path: KeyPath,
    pub diag: &'a Diagnostics,
    pub formatters: &'a Formatters,
}

impl<'de> serde::de::DeserializeSeed<'de> for ValuesSeed<'_> {
    type Value = RawValues;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let values = deserializer.deserialize_map(self)?;
        Ok(values)
    }
}

impl<'de> serde::de::Visitor<'de> for ValuesSeed<'_> {
    type Value = RawValues;

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = BTreeMap::new();

        while let Some(locale_key) = map.next_key::<Key>()? {
            let pushed_key = self.key_path.push_key(locale_key.clone());
            let value = map.next_value_seed(ValueOrSubkeysSeed {
                top_locale_name: &self.top_locale_name,
                key: &locale_key,
                key_path: &pushed_key,
                diag: self.diag,
                formatters: self.formatters,
            })?;
            values.insert(locale_key, value);
        }

        Ok(RawValues { values })
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a map of string keys and value either string or map"
        )
    }
}

#[derive(Clone, Copy)]
pub struct ValueOrSubkeysSeed<'a> {
    pub top_locale_name: &'a Key,
    pub key_path: &'a KeyPath,
    pub key: &'a Key,
    pub diag: &'a Diagnostics,
    pub formatters: &'a Formatters,
}

impl<'de> serde::de::DeserializeSeed<'de> for ValueOrSubkeysSeed<'_> {
    type Value = RawValueOrSubkeys;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> serde::de::Visitor<'de> for ValueOrSubkeysSeed<'_> {
    type Value = RawValueOrSubkeys;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let ctx = ParseContext {
            loc: Loc {
                key_path: self.key_path,
                locale: self.top_locale_name,
            },
            diag: self.diag,
            formatters: self.formatters,
            parse_fns: RawValue::DEFAULT_FNS,
        };

        let pv = RawValue::parse(&ctx, v);

        if let Ok(pv) = pv {
            Ok(RawValueOrSubkeys::Value(pv))
        } else {
            let dummy = Dummy::parse(v);
            Ok(RawValueOrSubkeys::Dummy(dummy))
        }
    }

    fn visit_bool<E>(self, v: bool) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RawValueOrSubkeys::Value(RawValue::Literal(
            RawLiteral::Bool(v),
        )))
    }

    fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RawValueOrSubkeys::Value(RawValue::Literal(
            RawLiteral::Signed(v),
        )))
    }

    fn visit_f64<E>(self, v: f64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RawValueOrSubkeys::Value(RawValue::Literal(
            RawLiteral::Float(v),
        )))
    }

    fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RawValueOrSubkeys::Value(RawValue::Literal(
            RawLiteral::Unsigned(v),
        )))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let map_de = MapAccessDeserializer::new(map);

        let seed = ValuesSeed {
            name: self.key.clone(),
            top_locale_name: self.top_locale_name.clone(),
            key_path: self.key_path.to_owned(),
            diag: self.diag,
            formatters: self.formatters,
        };

        seed.deserialize(map_de).map(RawValueOrSubkeys::Subkeys)
    }

    fn visit_unit<E>(self) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RawValueOrSubkeys::Defaulted)
    }

    // fn visit_seq<A>(mut self, map: A) -> Result<Self::Value, A::Error>
    // where
    //     A: serde::de::SeqAccess<'de>,
    // {
    //     self.diag.set_has_ranges();
    //     // nested ranges are not allowed, the code technically supports it,
    //     // but it's pointless and probably nobody will ever needs it.
    //     if std::mem::replace(&mut self.in_range, true) {
    //         return Err(serde::de::Error::custom(Error::NestedRanges));
    //     }
    //     let ranges = Ranges::from_serde_seq(map, self)?;

    //     let (invalid_fallback, fallback_count, should_have_fallback) =
    //         ranges.check_deserialization();

    //     if invalid_fallback {
    //         Err(serde::de::Error::custom(Error::InvalidFallback))
    //     } else if fallback_count > 1 {
    //         Err(serde::de::Error::custom(Error::MultipleFallbacks))
    //     } else if fallback_count == 0 && should_have_fallback {
    //         Err(serde::de::Error::custom(Error::MissingFallback(
    //             ranges.get_type(),
    //         )))
    //     } else {
    //         Ok(ParsedValue::Ranges(ranges))
    //     }
    // }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "either a string, a sequence of ranges or a map of subkeys"
        )
    }
}
