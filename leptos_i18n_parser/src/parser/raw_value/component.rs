use crate::{
    error::{Error, Result},
    parser::ParseContext,
    utils::{Key, Loc},
};

use super::{RawLiteral, RawValue};

#[derive(Debug, Clone, PartialEq)]
pub struct Component<V = RawValue, A = RawAttributes> {
    pub key: Key,
    pub inner: Option<Box<V>>,
    pub attributes: A,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct RawAttributes {
    pub attrs: Vec<RawAttribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawAttribute {
    pub key: String,
    pub value: Option<RawAttributeValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawAttributeValue {
    Literal(RawLiteral),
    Variable(Key),
}

pub struct ComponentFound<'a> {
    pub before: &'a str,
    pub after: &'a str,
    pub key: Key,
    pub inner: Option<&'a str>,
    pub attrs: &'a str,
}

struct OpeningTagFound<'a> {
    before: &'a str,
    after: &'a str,
    attrs: &'a str,
    key: &'a str,
    skipped: usize,
    self_closed: bool,
}

struct AttrKeyAndVal<'a> {
    key: &'a str,
    value: Option<RawAttributeValue>,
    rest: &'a str,
}

impl Component {
    pub fn parse(ctx: &ParseContext, s: &str) -> Option<Result<RawValue, ()>> {
        let component = match Self::find_component(s, ctx.loc) {
            Some(Ok(comp)) => comp,
            None => return None,
            Some(Err(err)) => {
                ctx.diag.emit_error(err);
                return Some(Err(()));
            }
        };

        let mut errored = false;

        let mut bloc = Vec::new();

        match RawValue::parse(ctx, component.before) {
            Ok(before) => bloc.push(before),
            Err(()) => errored = true,
        }

        let inner = if let Some(inner) = component.inner {
            match RawValue::parse(ctx, inner) {
                Ok(inner) => Some(Box::new(inner)),
                Err(()) => {
                    errored = true;
                    None
                }
            }
        } else {
            None
        };

        let attributes = match RawAttributes::parse(ctx, component.attrs) {
            Ok(attrs) => attrs,
            Err(()) => {
                errored = true;
                RawAttributes::default()
            }
        };

        let this = Component {
            key: component.key,
            inner,
            attributes,
        };

        bloc.push(RawValue::Component(this));

        match RawValue::parse(ctx, component.after) {
            Ok(RawValue::Bloc(mut next_bloc)) => bloc.append(&mut next_bloc),
            Ok(after) => bloc.push(after),
            Err(()) => errored = true,
        };

        if errored {
            Some(Err(()))
        } else {
            Some(Ok(RawValue::Bloc(bloc)))
        }
    }

    pub fn find_component<'a>(value: &'a str, loc: Loc<'_>) -> Option<Result<ComponentFound<'a>>> {
        let mut skip_sum = 0;

        loop {
            let opening_tag = Self::find_opening_tag(&value[skip_sum..])?;
            let key = Key::try_new_at(&format!("comp_{}", opening_tag.key), loc);
            let key = match key {
                Ok(key) => key,
                Err(err) => return Some(Err(err)),
            };

            // Calculate the absolute position of where this tag ends
            let tag_end = skip_sum + opening_tag.skipped;
            let before = &value[..skip_sum + opening_tag.before.len()];

            if opening_tag.self_closed {
                let comp_found = ComponentFound {
                    before,
                    after: opening_tag.after,
                    key,
                    inner: None,
                    attrs: opening_tag.attrs,
                };
                return Some(Ok(comp_found));
            }

            if let Some((between, after)) =
                Self::find_closing_tag(opening_tag.after, opening_tag.key)
            {
                let comp_found = ComponentFound {
                    before,
                    after,
                    key,
                    inner: Some(between),
                    attrs: opening_tag.attrs,
                };
                return Some(Ok(comp_found));
            }

            // No closing tag found - skip past this entire tag (including the tag itself)
            // so that the skipped tag text becomes part of the next iteration's "before"
            skip_sum = tag_end;
        }
    }

    fn find_opening_tag<'a>(s: &'a str) -> Option<OpeningTagFound<'a>> {
        let mut skipped = 0;

        loop {
            if s.len() - skipped < 4 {
                // Needs at least 4 chars for a component: <a/>
                return None;
            }

            let value = &s[skipped..];

            let open_idx = value.find('<')?;
            let close_idx = value[open_idx..].find('>')? + open_idx;

            let before = &value[..open_idx];
            let tag_content = &value[open_idx + 1..close_idx];
            let after = &value[close_idx + 1..];

            let (tag_content, self_closed) = if let Some(tc) = tag_content.strip_suffix('/') {
                (tc.trim(), true)
            } else {
                (tag_content.trim(), false)
            };

            skipped += close_idx + 1;

            if tag_content.is_empty() {
                continue;
            }

            let (key, attrs) = match tag_content.split_once(' ') {
                Some((key, attrs)) => (key, attrs.trim_start()),
                None => (tag_content, ""),
            };

            break Some(OpeningTagFound {
                before,
                after,
                attrs,
                key,
                skipped,
                self_closed,
            });
        }
    }

    fn find_closing_tag<'a>(value: &'a str, key: &str) -> Option<(&'a str, &'a str)> {
        let mut depth = 0usize;
        let mut search_start = 0;

        while let Some(rel_open) = value[search_start..].find('<') {
            let open_idx = search_start + rel_open;
            let Some(rel_close) = value[open_idx..].find('>') else {
                break;
            };
            let close_idx = open_idx + rel_close;

            let tag_content = value[open_idx + 1..close_idx].trim();
            search_start = close_idx + 1;

            if tag_content.ends_with('/') {
                // Self-closing tag, skip
                continue;
            }

            if let Some(closing_name) = tag_content.strip_prefix('/') {
                let closing_name = closing_name.trim_start();
                if closing_name == key {
                    if depth == 0 {
                        let before = &value[..open_idx];
                        let after = &value[close_idx + 1..];
                        return Some((before, after));
                    }
                    depth -= 1;
                }
            } else if tag_content == key {
                depth += 1;
            }
        }

        None
    }
}

impl RawAttributes {
    pub fn parse(ctx: &ParseContext, mut attrs: &str) -> Result<RawAttributes, ()> {
        let _ = ctx;
        let mut attributes = Vec::new();
        loop {
            let key_and_val = match Self::pop_key_and_value(ctx, attrs) {
                None => return Ok(RawAttributes { attrs: attributes }),
                Some(Err(rest)) => {
                    attrs = rest;
                    continue;
                }
                Some(Ok(v)) => v,
            };

            attrs = key_and_val.rest;

            attributes.push(RawAttribute {
                key: key_and_val.key.to_string(),
                value: key_and_val.value,
            });
        }
    }

    fn pop_key_and_value<'a>(
        ctx: &ParseContext,
        s: &'a str,
    ) -> Option<Result<AttrKeyAndVal<'a>, &'a str>> {
        let s = s.trim_start();
        if s.is_empty() {
            return None;
        }
        let (maybe_key, rest) = s
            .find(|c: char| c.is_whitespace() || c == '=')
            .and_then(|i| s.split_at_checked(i))
            .unwrap_or((s, ""));

        let key = Self::validate_key(ctx, maybe_key).ok();

        let rest = rest.trim_start();

        let stripped = rest.strip_prefix('=').map(str::trim_start);

        let (value, rest) = match stripped {
            Some("") => {
                ctx.diag.emit_error(Error::InvalidAttribute {
                    loc: ctx.into(),
                    attr_name: maybe_key.to_string(),
                    attr_value: String::new(),
                    err: "attribute have an '=' sign but no value afterward".to_string(),
                });
                return Some(Err(""));
            }
            Some(s) => match Self::pop_value(ctx, maybe_key, s) {
                Ok((value, rest)) => (Some(value), rest),
                Err(rest) => return Some(Err(rest)),
            },
            None => (None, rest),
        };

        let Some(key) = key else {
            return Some(Err(rest));
        };

        Some(Ok(AttrKeyAndVal { key, value, rest }))
    }

    fn pop_value<'a>(
        ctx: &ParseContext,
        key: &str,
        s: &'a str,
    ) -> Result<(RawAttributeValue, &'a str), &'a str> {
        if let Some(rest) = s.strip_prefix("true ") {
            Ok((RawAttributeValue::Literal(RawLiteral::Bool(true)), rest))
        } else if let Some(rest) = s.strip_prefix("false ") {
            Ok((RawAttributeValue::Literal(RawLiteral::Bool(false)), rest))
        } else if let Some(rest) = s.strip_prefix('"') {
            match Self::parse_attribute_str(ctx, key, rest) {
                Ok((v, rest)) => Ok((
                    RawAttributeValue::Literal(RawLiteral::String(v.to_string())),
                    rest,
                )),
                Err(rest) => Err(rest),
            }
        } else if s.starts_with(Self::is_num()) {
            let (num, rest) = s.split_once(char::is_whitespace).unwrap_or((s, ""));
            match Self::parse_num(num) {
                Some(num) => Ok((RawAttributeValue::Literal(num), rest)),
                None => {
                    ctx.diag.emit_error(Error::InvalidAttribute {
                        loc: ctx.into(),
                        attr_value: num.to_string(),
                        attr_name: key.to_string(),
                        err: "value appears to be a number, but can't be parsed as one".to_string(),
                    });
                    Err(rest)
                }
            }
        } else if let Some(rest) = s.strip_prefix("{{") {
            match rest.split_once("}}") {
                Some((key, rest)) => {
                    match Key::try_new_at(&format!("var_{}", key.trim()), ctx.loc) {
                        Ok(key) => Ok((RawAttributeValue::Variable(key), rest)),
                        Err(err) => {
                            ctx.diag.emit_error(err);
                            Err(rest)
                        }
                    }
                }
                None => {
                    ctx.diag.emit_error(Error::InvalidAttribute {
                        loc: ctx.into(),
                        attr_value: rest.to_string(),
                        attr_name: key.to_string(),
                        err: "unterminated variable".to_string(),
                    });
                    Err("")
                }
            }
        } else {
            ctx.diag.emit_error(Error::InvalidAttribute {
                loc: ctx.into(),
                attr_value: s.to_string(),
                attr_name: key.to_string(),
                err: "invalid argument (expect string, number, boolean or variable)".to_string(),
            });
            Err("")
        }
    }

    fn parse_attribute_str_inner(s: &str) -> Option<(&str, &str)> {
        let mut escaped = false;
        s.char_indices()
            .find_map(|(i, c)| {
                if core::mem::replace(&mut escaped, false) {
                    None
                } else if c == '\\' {
                    escaped = true;
                    None
                } else if c == '"' {
                    Some(i)
                } else {
                    None
                }
            })
            .and_then(|i| Some((s.get(..i)?, s.get(i + 1..)?)))
    }

    fn parse_attribute_str<'a>(
        ctx: &ParseContext,
        key: &str,
        s: &'a str,
    ) -> Result<(&'a str, &'a str), &'a str> {
        match Self::parse_attribute_str_inner(s) {
            Some(v) => Ok(v),
            None => {
                let mut attr_value = String::with_capacity(s.len() + 1);
                attr_value.push('"');
                attr_value.push_str(s);
                ctx.diag.emit_error(Error::InvalidAttribute {
                    loc: ctx.into(),
                    attr_value,
                    attr_name: key.to_string(),
                    err: "unterminated string".to_string(),
                });
                Err("")
            }
        }
    }

    fn is_num() -> impl FnMut(char) -> bool {
        let mut first_dot = true;
        move |c| char::is_ascii_digit(&c) || (c == '.' && core::mem::replace(&mut first_dot, false))
    }

    fn parse_num(num: &str) -> Option<RawLiteral> {
        if let Ok(n) = num.parse() {
            Some(RawLiteral::Unsigned(n))
        } else if let Ok(n) = num.parse() {
            Some(RawLiteral::Signed(n))
        } else if let Ok(n) = num.parse() {
            Some(RawLiteral::Float(n))
        } else {
            None
        }
    }

    fn validate_key<'a>(ctx: &ParseContext, key: &'a str) -> Result<&'a str, ()> {
        if key.chars().all(|c| c.is_alphabetic() || c == '_') {
            Ok(key)
        } else {
            ctx.diag.emit_error(Error::InvalidAttributeName {
                loc: ctx.into(),
                value: key.to_string(),
            });

            Err(())
        }
    }
}
