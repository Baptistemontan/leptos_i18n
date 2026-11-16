use std::collections::{BTreeMap, btree_map::Entry};

use serde::de::{DeserializeSeed, MapAccess, Visitor};

use crate::{
    error::{Error, Result, Warning},
    parser::{
        ParseContext,
        raw_value::{RawLiteral, RawValue},
    },
    utils::{Key, KeyPath, Loc, Location},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ForeignKey {
    pub target_location: Location,
    pub args: BTreeMap<String, RawValue>,
}

impl ForeignKey {
    fn new_inner(target_location: Location, args: BTreeMap<String, RawValue>) -> Self {
        ForeignKey {
            target_location,
            args,
        }
    }

    pub fn parse(ctx: &ParseContext, s: &str) -> Option<Result<RawValue, ()>> {
        let (before, rest) = s.split_once("$t(")?;
        let next_split = rest.find([',', ')'])?;
        let keypath = rest.get(..next_split)?;
        let sep = rest[next_split..].chars().next()?;
        let after = rest.get(next_split + sep.len_utf8()..)?;
        let target_key_path = match Self::parse_key_path(keypath, ctx.loc) {
            Ok(kp) => kp,
            Err(err) => {
                ctx.diag.emit_error(err);
                return Some(Err(()));
            }
        };

        let (args, after) = if sep == ',' {
            match Self::parse_args(after, &target_key_path, ctx) {
                Ok((args, after)) => (Some(args), after),
                Err(after) => (None, after),
            }
        } else {
            (Some(BTreeMap::new()), after)
        };

        let target_location = Location {
            key_path: target_key_path,
            locale: ctx.loc.locale.clone(),
        };

        let this = args.map(|args| RawValue::ForeignKey(Self::new_inner(target_location, args)));

        let before = RawValue::parse(ctx, before);
        let after = RawValue::parse(ctx, after);

        match (before, this, after) {
            (Ok(before), Some(this), Ok(after)) => {
                Some(Ok(RawValue::Bloc(vec![before, this, after])))
            }
            _ => Some(Err(())),
        }
    }

    fn parse_key_path(path: &str, loc: Loc) -> Result<KeyPath> {
        let (ns, path) = if let Some((namespace, rest)) = path.split_once(':') {
            let namespace = Key::try_new_at(namespace, loc)?;

            (Some(namespace), rest)
        } else {
            (None, path)
        };
        let mut key_path = Vec::new();
        for key in path.split('.') {
            let key = Key::try_new_at(key, loc)?;
            key_path.push(key);
        }

        Ok(KeyPath::new_from_path(ns, key_path))
    }

    /// Deserialize the arguments in order, so a duplicated argument or two arguments resolving to the same
    /// identifier (`a-b` and `a_b`) can be reported. The first one is kept.
    fn deserialize_args(
        ctx: &ParseContext,
        fk_target: &KeyPath,
        s: &str,
    ) -> serde_json::Result<BTreeMap<String, (RawLiteral, String)>> {
        struct ArgsSeed<'a, 'b> {
            ctx: &'a ParseContext<'b>,
            fk_target: &'a KeyPath,
        }

        impl<'de> Visitor<'de> for ArgsSeed<'_, '_> {
            /// normalized name -> (value, name as written)
            type Value = BTreeMap<String, (RawLiteral, String)>;

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut args = BTreeMap::new();
                while let Some((name, value)) = map.next_entry::<String, RawLiteral>()? {
                    let name = name.trim().to_string();
                    match args.entry(name.replace('-', "_")) {
                        Entry::Vacant(entry) => {
                            entry.insert((value, name));
                        }
                        Entry::Occupied(entry) => {
                            let (_, first_name) = entry.get();
                            let warning = if *first_name == name {
                                Warning::DuplicateFkArgs {
                                    loc: self.ctx.into(),
                                    fk_target: self.fk_target.clone(),
                                    arg_name: name,
                                }
                            } else {
                                Warning::CollidingFkArgs {
                                    loc: self.ctx.into(),
                                    fk_target: self.fk_target.clone(),
                                    arg1: first_name.clone(),
                                    arg2: name,
                                    ident_repr: entry.key().clone(),
                                }
                            };
                            self.ctx.diag.emit_warning(warning);
                        }
                    }
                }
                Ok(args)
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a map of foreign key arguments")
            }
        }

        impl<'de> DeserializeSeed<'de> for ArgsSeed<'_, '_> {
            type Value = BTreeMap<String, (RawLiteral, String)>;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_map(self)
            }
        }

        let mut deserializer = serde_json::Deserializer::from_str(s);
        let args = ArgsSeed { ctx, fk_target }.deserialize(&mut deserializer)?;
        deserializer.end()?;
        Ok(args)
    }

    /// The arguments are keyed by the normalized variable name (`var_a_b` for `a-b`),
    /// look them up with the variable key's `ident_repr`.
    fn parse_args_inner(
        ctx: &ParseContext,
        fk_target: &KeyPath,
        s: &str,
    ) -> Result<BTreeMap<String, RawValue>, ()> {
        let args = match Self::deserialize_args(ctx, fk_target, s) {
            Ok(args) => args,
            Err(err) => {
                ctx.diag.emit_error(Error::InvalidForeignKeyArgs {
                    loc: ctx.into(),
                    err,
                });
                return Err(());
            }
        };

        let mut parsed_args = BTreeMap::new();

        let mut errored = false;

        for (key, (arg, _)) in args {
            let parsed_value = match arg {
                RawLiteral::String(s) => match RawValue::parse(ctx, &s) {
                    Ok(rv) => rv,
                    Err(()) => {
                        errored = true;
                        continue;
                    }
                },
                other => RawValue::Literal(other),
            };
            let key = format!("var_{key}");
            parsed_args.insert(key, parsed_value);
        }

        if errored { Err(()) } else { Ok(parsed_args) }
    }

    fn parse_args<'a>(
        s: &'a str,
        fk_target: &KeyPath,
        ctx: &ParseContext,
    ) -> Result<(BTreeMap<String, RawValue>, &'a str), &'a str> {
        let mut depth = 0usize;
        let mut index = 0usize;

        for (i, c) in s.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth = match depth.checked_sub(1) {
                        Some(v) => v,
                        None => {
                            ctx.diag.emit_error(Error::UnexpectedToken {
                                loc: ctx.into(),
                                message: "malformed foreign key".to_string(),
                            });
                            return Err("");
                        }
                    };
                    if depth == 0 {
                        index = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        let (before, after) = s.split_at(index + '}'.len_utf8());

        let Some(after) = after.trim_start().strip_prefix(')') else {
            ctx.diag.emit_error(Error::UnexpectedToken {
                loc: ctx.into(),
                message: "malformed foreign key".to_string(),
            });
            return Err("");
        };

        match Self::parse_args_inner(ctx, fk_target, before) {
            Ok(args) => Ok((args, after)),
            Err(()) => Err(after),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{error::Diagnostics, formatters::Formatters, options::LocaleName, parser::ParseFn};

    use super::*;

    struct TestResult {
        value: Option<RawValue>,
        locale: LocaleName,
        diag: Diagnostics,
    }

    fn test_util(s: &str, locale: &str, parse_fns: &[ParseFn]) -> TestResult {
        let key_path = KeyPath::new(None);
        let locale = LocaleName {
            key: Key::new(locale).unwrap(),
            loc_id: Rc::new(locale.parse().unwrap()),
        };
        let formatters = Formatters::new();
        let diag = Diagnostics::new();

        let ctx = ParseContext {
            loc: Loc {
                key_path: &key_path,
                locale: &locale,
            },
            formatters: &formatters,
            diag: &diag,
            parse_fns,
        };

        let value = parse_fns
            .iter()
            .find_map(|parse_fn| parse_fn(&ctx, s).transpose().unwrap());

        TestResult {
            value,
            locale,
            diag,
        }
    }

    #[test]
    fn test_parsing_namespaces() {
        let TestResult {
            value,
            locale,
            diag,
        } = test_util(
            "before $t(second_namespace:common_key) after",
            "fr",
            &[ForeignKey::parse],
        );
        let value = value.unwrap();

        assert_eq!(
            value,
            RawValue::Bloc(vec![
                RawValue::Literal(RawLiteral::String("before ".to_string())),
                RawValue::ForeignKey(ForeignKey {
                    target_location: Location {
                        key_path: KeyPath {
                            namespace: Some(Key::new("second_namespace").unwrap()),
                            path: vec![Key::new("common_key").unwrap()]
                        },
                        locale
                    },
                    args: BTreeMap::new()
                }),
                RawValue::Literal(RawLiteral::String(" after".to_string()))
            ])
        );

        assert!(diag.warnings().is_empty());
        assert!(diag.errors().is_empty());
    }
}
