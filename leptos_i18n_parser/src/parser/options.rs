use super::{DEFAULT_LOCALES_PATH, ValuesSeed};
use crate::{
    error::{Error, Result, SerdeError},
    formatters::{Formatter, Formatters},
    parser::RawValues,
    utils::Key,
};
use parser::Parser;
use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt::Debug,
    fmt::Display,
    panic::Location,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

#[derive(Debug, Clone)]
pub struct LocaleName {
    pub key: Key,
    pub loc_id: Rc<icu_locale::Locale>,
}

impl Ord for LocaleName {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl PartialOrd for LocaleName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for LocaleName {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for LocaleName {}

impl Display for LocaleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&*self.loc_id, f)
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Config {
    pub default_locale: Key,
    pub locales: Vec<LocaleName>,
    pub locales_path: Cow<'static, Path>,
    pub namespaces: Vec<Key>,
    pub extensions: BTreeMap<Key, Key>,
    pub options: ParseOptions,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ParseOptions {
    pub file_format: FileFormat,
    /// Don't emit the warnings for missing keys and surplus keys.
    pub suppress_key_warnings: bool,
    pub formatters: Formatters,
}

#[derive(Clone, Default)]
#[non_exhaustive]
pub enum FileFormat {
    #[default]
    Json,
    Json5,
    Yaml,
    Toml,
    Custom(Arc<dyn Parser>),
}

impl Debug for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileFormat::Json => f.write_str("Json"),
            FileFormat::Json5 => f.write_str("Json5"),
            FileFormat::Yaml => f.write_str("Yaml"),
            FileFormat::Toml => f.write_str("Toml"),
            FileFormat::Custom(..) => f.debug_tuple("Custom").finish(),
        }
    }
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseOptions {
    pub fn new() -> Self {
        ParseOptions {
            file_format: FileFormat::Json,
            suppress_key_warnings: false,
            formatters: Formatters::new(),
        }
    }

    pub fn file_format(self, file_format: FileFormat) -> Self {
        Self {
            file_format,
            ..self
        }
    }

    /// Don't emit the warnings for missing keys and surplus keys.
    pub fn suppress_key_warnings(self, suppress_key_warnings: bool) -> Self {
        Self {
            suppress_key_warnings,
            ..self
        }
    }

    pub fn with_custom_parser<P: Parser>(self, parser: P) -> Self {
        Self {
            file_format: FileFormat::Custom(Arc::new(parser)),
            ..self
        }
    }

    pub fn add_formatter<F: Formatter>(mut self, formatter: F) -> Self {
        self.formatters
            .insert_formatter(formatter)
            .expect("tried to overwrite an existing formatter.");
        self
    }
}

impl Config {
    pub fn new(default_locale: &str) -> Result<Self> {
        let default_locale = Self::new_locale_name(default_locale)?;
        Ok(Config {
            default_locale: default_locale.key.clone(),
            locales: vec![default_locale],
            locales_path: Cow::Borrowed(DEFAULT_LOCALES_PATH.as_ref()),
            namespaces: vec![],
            // translations_uri: None,
            extensions: BTreeMap::new(),
            options: ParseOptions::default(),
        })
    }

    fn new_locale_name(name: &str) -> Result<LocaleName> {
        let loc_id = name
            .parse()
            .map(Rc::new)
            .map_err(|err| Error::InvalidLocale {
                locale: name.to_string(),
                err,
            })?;
        let default_locale_key = Key::try_new(name)?;
        Ok(LocaleName {
            key: default_locale_key.clone(),
            loc_id,
        })
    }

    #[track_caller]
    fn add_locale_inner(&mut self, name: &str) -> Result<()> {
        let loc = Self::new_locale_name(name)?;
        if self.locales.contains(&loc) {
            let location = Location::caller();
            println!(
                "cargo::warning=locale \"{name}\" already present, re-added at {location}, it is skipped",
            );
        } else {
            self.locales.push(loc);
        }
        Ok(())
    }

    #[track_caller]
    pub fn add_locale(mut self, locale: &str) -> Result<Self> {
        self.add_locale_inner(locale)?;
        Ok(self)
    }

    #[track_caller]
    pub fn add_locales<T: AsRef<str>>(
        mut self,
        locales: impl IntoIterator<Item = T>,
    ) -> Result<Self> {
        for locale in locales {
            self = self.add_locale(locale.as_ref())?;
        }
        Ok(self)
    }

    pub fn locales_path(self, path: impl ToPathCow<'static>) -> Self {
        Self {
            locales_path: path.into_cow(),
            ..self
        }
    }

    pub fn add_namespace(mut self, namespace: &str) -> Result<Self> {
        self.namespaces.push(Key::try_new(namespace)?);
        Ok(self)
    }

    pub fn add_namespaces<T: AsRef<str>>(
        mut self,
        locales: impl IntoIterator<Item = T>,
    ) -> Result<Self> {
        for locale in locales {
            self = self.add_namespace(locale.as_ref())?;
        }
        Ok(self)
    }

    #[track_caller]
    pub fn extend_locale(mut self, locale_to_extend: &str, inherit_from: &str) -> Result<Self> {
        #[track_caller]
        fn check_if_present(locales: &[LocaleName], loc: &Key) -> Result<()> {
            let is_present = locales.iter().find(|l| l.key == *loc).is_some();
            if !is_present {
                Err(Error::UnknownLocaleInInherit {
                    loc: Location::caller(),
                    locale: loc.to_string(),
                }
                .into())
            } else {
                Ok(())
            }
        }

        let loc_to_ext = Key::try_new(locale_to_extend)?;
        let inh_from = Key::try_new(inherit_from)?;

        if loc_to_ext == self.default_locale {
            return Err(Error::DefaultLocaleCantInherit {
                loc: Location::caller(),
            }
            .into());
        }

        check_if_present(&self.locales, &loc_to_ext)?;
        check_if_present(&self.locales, &inh_from)?;

        if loc_to_ext == inh_from {
            // TODO: do we allow inherit from self ? cyclics inheritance are allowed so does this count ?
        }

        if let Some(old_inh) = self.extensions.insert(loc_to_ext, inh_from) {
            let location = Location::caller();
            println!(
                "cargo::warning=locale \"{locale_to_extend}\" already inherit from \"{old_inh}\", overwritten at {location}"
            );
        }
        Ok(self)
    }

    /// Returns the locale `locale` inherits its missing keys from:
    /// its explicit extension (see `extend_locale`), else its base locale if declared
    /// (`language[-region][-anything-else]` inherits from `language[-region]`, then from `language`).
    ///
    /// `None` means it falls back to the default locale (always the case for the default locale itself).
    pub fn inherits_from(&self, locale: &Key) -> Option<&Key> {
        if *locale == self.default_locale {
            return None;
        }
        self.extensions
            .get(locale)
            .or_else(|| self.find_base_locale(locale))
    }

    /// Finds the base locale of `locale` among the declared locales.
    fn find_base_locale(&self, locale: &Key) -> Option<&Key> {
        let icu_locale = &self.locales.iter().find(|l| l.key == *locale)?.loc_id;

        // just a language, no base locale.
        if icu_locale.extensions.is_empty()
            && icu_locale.id.variants.is_empty()
            && icu_locale.id.script.is_none()
            && icu_locale.id.region.is_none()
        {
            return None;
        }

        let mut looking_for = icu_locale::Locale {
            extensions: Default::default(),
            id: icu_locale::LanguageIdentifier {
                language: icu_locale.id.language,
                region: icu_locale.id.region,
                script: None,
                variants: Default::default(),
            },
        };

        let find = |looking_for: &icu_locale::Locale| {
            self.locales
                .iter()
                .find(|l| *l.loc_id == *looking_for)
                .map(|l| &l.key)
        };

        // with extensions, variants or a script, first look for the same language and region without them
        if (!icu_locale.extensions.is_empty()
            || !icu_locale.id.variants.is_empty()
            || icu_locale.id.script.is_some())
            && let Some(found) = find(&looking_for)
        {
            return Some(found);
        }

        // if not found, relax the region
        looking_for.id.region = None;
        find(&looking_for)
    }

    pub fn parse_options(self, options: ParseOptions) -> Self {
        Self { options, ..self }
    }
}

impl FileFormat {
    pub fn get_files_exts(&self) -> &'static [&'static str] {
        match self {
            FileFormat::Json => &["json"],
            FileFormat::Json5 => &["json5"],
            FileFormat::Yaml => &["yaml", "yml"],
            FileFormat::Toml => &["toml"],
            FileFormat::Custom(parser) => parser.file_extensions(),
        }
    }

    pub fn deserialize(
        &self,
        locale_file: &str,
        path: Option<&Path>,
        seed: ValuesSeed,
    ) -> Result<RawValues, SerdeError> {
        match self {
            FileFormat::Json => de_json(locale_file, seed),
            FileFormat::Json5 => de_json5(locale_file, seed),
            FileFormat::Yaml => de_yaml(locale_file, seed),
            FileFormat::Toml => de_toml(locale_file, seed),
            FileFormat::Custom(parser) => parser::de_custom(&**parser, locale_file, path, seed),
        }
    }
}

fn de_json(locale_file: &str, seed: ValuesSeed) -> Result<RawValues, SerdeError> {
    let mut deserializer = serde_json::Deserializer::from_str(locale_file);
    serde::de::DeserializeSeed::deserialize(seed, &mut deserializer).map_err(SerdeError::Json)
}

fn de_json5(locale_file: &str, seed: ValuesSeed) -> Result<RawValues, SerdeError> {
    let mut deserializer = json5::Deserializer::from_str(locale_file);
    serde::de::DeserializeSeed::deserialize(seed, &mut deserializer).map_err(SerdeError::Json5)
}

fn de_yaml(locale_file: &str, seed: ValuesSeed) -> Result<RawValues, SerdeError> {
    let deserializer = serde_yaml::Deserializer::from_str(locale_file);
    serde::de::DeserializeSeed::deserialize(seed, deserializer).map_err(SerdeError::Yaml)
}

fn de_toml(locale_file: &str, seed: ValuesSeed) -> Result<RawValues, SerdeError> {
    let deserializer = toml::Deserializer::parse(locale_file).map_err(SerdeError::Toml)?;
    serde::de::DeserializeSeed::deserialize(seed, deserializer).map_err(SerdeError::Toml)
}

pub mod parser {
    use std::path::Path;

    use crate::{error::SerdeError, parser::ValuesSeed, parsing::RawValues};

    pub struct Seed<'a>(ValuesSeed<'a>);

    impl<'de> serde::de::DeserializeSeed<'de> for Seed<'_> {
        type Value = Value;
        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            serde::de::DeserializeSeed::deserialize(self.0, deserializer).map(Value)
        }
    }

    pub struct Value(RawValues);

    pub trait Parser: 'static {
        fn deserialize(
            &self,
            file: &str,
            path: Option<&Path>,
            seed: Seed,
        ) -> Result<Value, SerdeError>;

        fn file_extensions(&self) -> &'static [&'static str];
    }

    pub(crate) fn de_custom(
        parser: &dyn Parser,
        file: &str,
        path: Option<&Path>,
        seed: ValuesSeed,
    ) -> Result<RawValues, SerdeError> {
        let seed = Seed(seed);
        let value = parser.deserialize(file, path, seed)?;
        Ok(value.0)
    }
}

pub trait ToPathCow<'a> {
    fn into_cow(self) -> Cow<'a, Path>;
}

impl<'a> ToPathCow<'a> for &'a str {
    fn into_cow(self) -> Cow<'a, Path> {
        Cow::Borrowed(self.as_ref())
    }
}

impl<'a> ToPathCow<'a> for &'a Path {
    fn into_cow(self) -> Cow<'a, Path> {
        Cow::Borrowed(self)
    }
}

impl<'a> ToPathCow<'a> for String {
    fn into_cow(self) -> Cow<'a, Path> {
        Cow::Owned(self.into())
    }
}

impl<'a> ToPathCow<'a> for PathBuf {
    fn into_cow(self) -> Cow<'a, Path> {
        Cow::Owned(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config::new("en")
            .unwrap()
            .add_locales([
                "fr",
                "en-US",
                "fr-FR",
                "fr-FR-u-ca-buddhist",
                "fr-u-ca-buddhist",
                "en-Latn-US-Valencia",
                "en-Latn-US-Valencia-u-ca-buddhist",
                "en-Latn-US-u-ca-buddhist",
                "en-Valencia",
                "en-Latn",
            ])
            .unwrap()
    }

    fn inherits_from(cfg: &Config, locale: &str) -> Option<String> {
        cfg.inherits_from(&Key::new(locale).unwrap())
            .map(|key| key.to_string())
    }

    #[test]
    fn base_locale_inheritance() {
        let cfg = config();
        let cases = [
            ("fr-FR-u-ca-buddhist", Some("fr-FR")),
            ("fr-u-ca-buddhist", Some("fr")),
            ("fr-FR", Some("fr")),
            ("fr", None),
            ("en-Latn-US-Valencia", Some("en-US")),
            ("en-Latn-US-Valencia-u-ca-buddhist", Some("en-US")),
            ("en-Latn-US-u-ca-buddhist", Some("en-US")),
            ("en-Valencia", Some("en")),
            ("en-Latn", Some("en")),
            ("en-US", Some("en")),
            // default locale
            ("en", None),
        ];
        for (locale, expected) in cases {
            assert_eq!(inherits_from(&cfg, locale).as_deref(), expected, "{locale}");
        }
    }

    #[test]
    fn explicit_extension_takes_precedence() {
        let cfg = config().extend_locale("fr-FR", "en-US").unwrap();
        assert_eq!(inherits_from(&cfg, "fr-FR").as_deref(), Some("en-US"));
        // others are unchanged
        assert_eq!(
            inherits_from(&cfg, "fr-FR-u-ca-buddhist").as_deref(),
            Some("fr-FR")
        );
    }
}
