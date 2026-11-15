use super::RawValue;
use crate::{
    error::{Result, Warning},
    formatters::VarBound,
    parser::ParseContext,
    utils::Key,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub key: Key,
    pub bound: VarBound,
}

impl Variable {
    pub fn parse(ctx: &ParseContext, s: &str) -> Option<Result<RawValue, ()>> {
        let (before, this, after) = Self::find_valid_variable(ctx, s)?;

        let before = RawValue::parse(ctx, before);
        let after = RawValue::parse(ctx, after);

        match (before, this, after) {
            (Ok(before), Some(this), Ok(after)) => Some(Ok(RawValue::Bloc(vec![
                before,
                RawValue::Variable(this),
                after,
            ]))),
            _ => Some(Err(())),
        }
    }

    pub fn actual_name(&self) -> &str {
        self.key
            .name
            .strip_prefix("var_")
            .expect("a variable name should start with var_")
    }

    fn find_valid_variable<'a>(
        ctx: &ParseContext,
        s: &'a str,
    ) -> Option<(&'a str, Option<Self>, &'a str)> {
        let (before, rest) = s.split_once("{{")?;
        // TODO: maybe error on unfinished variable ?
        let (ident, after) = rest.split_once("}}")?;

        let ident = ident.trim();

        let this = if let Some((ident, s)) = ident.split_once(',') {
            let bound = Self::parse_formatter(ctx, s);
            let ident = ident.trim();
            match Key::try_new_at(&format!("var_{ident}"), ctx.loc) {
                Ok(key) => Some(Variable { key, bound }),
                Err(err) => {
                    ctx.diag.emit_error(err);
                    None
                }
            }
        } else {
            match Key::try_new_at(&format!("var_{ident}"), ctx.loc) {
                Ok(key) => Some(Variable {
                    key,
                    bound: VarBound::None,
                }),
                Err(err) => {
                    ctx.diag.emit_error(err);
                    None
                }
            }
        };

        Some((before, this, after))
    }

    fn parse_formatter(ctx: &ParseContext, s: &str) -> VarBound {
        let (name, args) = Self::parse_formatter_args(ctx, s);
        ctx.formatters.parse(ctx, name, &args)
    }

    #[allow(clippy::type_complexity)]
    fn parse_formatter_args<'a>(
        ctx: &ParseContext,
        s: &'a str,
    ) -> (&'a str, Vec<(&'a str, Option<&'a str>)>) {
        let Some((name, rest)) = s.split_once('(') else {
            return (s.trim(), vec![]);
        };
        let Some((args, rest)) = rest.rsplit_once(')') else {
            return (s.trim(), vec![]);
        };

        let r = rest.trim();
        if !r.is_empty() {
            ctx.diag
                .emit_warning(Warning::UnexpectedCharsAfterFormatter {
                    loc: ctx.into(),
                    formatter_name: name.to_string(),
                    chars: r.to_string(),
                });
        }

        let args = args.split(';').map(|s| {
            s.split_once(':')
                .map(|(a, b)| (a.trim(), Some(b.trim())))
                .unwrap_or((s.trim(), None))
        });

        (name.trim(), args.collect())
    }
}
