use std::ops::Not;

use quote::{quote, ToTokens};

use super::error::{Error, Result};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Plural {
    Exact(i64),
    Range {
        start: Option<i64>,
        // originaly this was a Bound<i64>, but excluded ranges are unstable
        // so always output included end bound
        end: Option<i64>,
    },
    Fallback,
}

impl Plural {
    pub fn new(locale_name: &str, locale_key: &str, s: &str) -> Result<Self> {
        let parse = |s: &str| {
            s.parse::<i64>().map_err(|_| Error::InvalidPlural {
                locale_name: locale_name.to_string(),
                locale_key: locale_key.to_string(),
                plural: s.to_string(),
            })
        };
        let s = s.trim();
        if s == "_" {
            return Ok(Self::Fallback);
        };
        if let Some((start, end)) = s.split_once("..") {
            let start = start.trim();
            let start = start.is_empty().not().then(|| parse(start)).transpose()?;
            let end = end.trim();
            let end = if end.is_empty() {
                None
            } else if let Some(end) = end.strip_prefix('=').map(str::trim_start) {
                Some(parse(end)?)
            } else {
                let end = parse(end)?;
                end.checked_sub(1)
            };

            Ok(Self::Range { start, end })
        } else {
            parse(s).map(Self::Exact)
        }
    }
}

impl ToTokens for Plural {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match self {
            Plural::Exact(num) => quote!(#num),
            Plural::Range {
                start,
                end: Some(end),
            } => {
                quote!(#start..=#end)
            }
            Plural::Range { start, end: None } => {
                quote!(#start..)
            }
            Plural::Fallback => quote!(_),
        }
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        Self::to_token_stream(self).to_tokens(tokens)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PluralSeed<'a> {
    pub locale_name: &'a str,
    pub locale_key: &'a str,
}

impl<'de> serde::de::DeserializeSeed<'de> for PluralSeed<'_> {
    type Value = Plural;
    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de> serde::de::Visitor<'de> for PluralSeed<'_> {
    type Value = Plural;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "a string representing either a range, an integer, or a fallback \"_\""
        )
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        // return Err(E::custom(format!("{:?}", v.as_bytes())));
        Plural::new(self.locale_name, self.locale_key, v).map_err(E::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact() {
        let plural = Plural::new("", "", "0").unwrap();

        assert_eq!(plural, Plural::Exact(0));
    }

    #[test]
    fn test_fallback() {
        let plural = Plural::new("", "", "_").unwrap();

        assert_eq!(plural, Plural::Fallback);
    }

    #[test]
    fn test_range() {
        let plural = Plural::new("", "", "0..6").unwrap();

        assert_eq!(
            plural,
            Plural::Range {
                start: Some(0),
                end: Some(5) // see field comment for why it's 5
            }
        );
    }

    #[test]
    fn test_range_unbounded_end() {
        let plural = Plural::new("", "", "0..").unwrap();

        assert_eq!(
            plural,
            Plural::Range {
                start: Some(0),
                end: None
            }
        );
    }

    #[test]
    fn test_range_included_end() {
        let plural = Plural::new("", "", "0..=6").unwrap();

        assert_eq!(
            plural,
            Plural::Range {
                start: Some(0),
                end: Some(6)
            }
        );
    }

    #[test]
    fn test_range_unbounded_start() {
        let plural = Plural::new("", "", "..=6").unwrap();

        assert_eq!(
            plural,
            Plural::Range {
                start: None,
                end: Some(6)
            }
        );
    }

    #[test]
    fn test_range_full() {
        let plural = Plural::new("", "", "..").unwrap();

        assert_eq!(
            plural,
            Plural::Range {
                start: None,
                end: None
            }
        );
    }
}
