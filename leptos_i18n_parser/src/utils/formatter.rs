use std::cell::Cell;

use tinystr::{tinystr, TinyAsciiStr};

thread_local! {
    pub(crate) static SKIP_ICU_CFG: Cell<bool> = const { Cell::new(false) };
}

pub(crate) struct SkipIcuCfgGuard(());

impl SkipIcuCfgGuard {
    pub fn new(skip_icu_cfg: bool) -> Self {
        SKIP_ICU_CFG.set(skip_icu_cfg);
        SkipIcuCfgGuard(())
    }
}

impl Drop for SkipIcuCfgGuard {
    fn drop(&mut self) {
        SKIP_ICU_CFG.set(false);
    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Formatter {
    #[default]
    None,
    Number(GroupingStrategy),
    Date(DateLength),
    Time(TimeLength),
    DateTime(DateLength, TimeLength),
    List(ListType, ListStyle),
    Currency(CurrencyWidth, CurrencyCode),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CurrencyCode(pub TinyAsciiStr<3>);

impl Default for CurrencyCode {
    fn default() -> Self {
        Self(tinystr!(3, "USD"))
    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum CurrencyWidth {
    #[default]
    Short,
    Narrow,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum GroupingStrategy {
    #[default]
    Auto,
    Never,
    Always,
    Min2,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateLength {
    Full,
    Long,
    #[default]
    Medium,
    Short,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeLength {
    Full,
    Long,
    Medium,
    #[default]
    Short,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListType {
    And,
    Or,
    #[default]
    Unit,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListStyle {
    #[default]
    Wide,
    Short,
    Narrow,
}

impl Formatter {
    pub fn from_name_and_args<'a, S: PartialEq + PartialEq<&'a str> + ToString>(
        name: S,
        args: Option<&[(S, S)]>,
    ) -> Result<Option<Formatter>, Formatter> {
        if name == "currency" {
            let formatter = Formatter::Currency(
                CurrencyWidth::from_args(args),
                CurrencyCode::from_args(args),
            );
            if cfg!(feature = "format_currency") || SKIP_ICU_CFG.get() {
                Ok(Some(formatter))
            } else {
                Err(formatter)
            }
        } else if name == "number" {
            if cfg!(feature = "format_nums") || SKIP_ICU_CFG.get() {
                Ok(Some(Formatter::Number(GroupingStrategy::from_args(args))))
            } else {
                Err(Formatter::Number(GroupingStrategy::from_args(args)))
            }
        } else if name == "datetime" {
            let formatter =
                Formatter::DateTime(DateLength::from_args(args), TimeLength::from_args(args));
            if cfg!(feature = "format_datetime") || SKIP_ICU_CFG.get() {
                Ok(Some(formatter))
            } else {
                Err(formatter)
            }
        } else if name == "date" {
            let formatter = Formatter::Date(DateLength::from_args(args));
            if cfg!(feature = "format_datetime") || SKIP_ICU_CFG.get() {
                Ok(Some(formatter))
            } else {
                Err(formatter)
            }
        } else if name == "time" {
            let formatter = Formatter::Time(TimeLength::from_args(args));
            if cfg!(feature = "format_datetime") || SKIP_ICU_CFG.get() {
                Ok(Some(formatter))
            } else {
                Err(formatter)
            }
        } else if name == "list" {
            let formatter = Formatter::List(ListType::from_args(args), ListStyle::from_args(args));
            if cfg!(feature = "format_list") || SKIP_ICU_CFG.get() {
                Ok(Some(formatter))
            } else {
                Err(formatter)
            }
        } else {
            Ok(None)
        }
    }

    pub fn err_message(&self) -> &'static str {
        match self {
            Formatter::None => "",
            Formatter::Number(_) => "Formatting numbers is not enabled, enable the \"format_nums\" feature to do so",
            Formatter::Currency(_, _) => "Formatting currencies is not enabled, enable the \"format_currency\" feature to do so",
            Formatter::Date(_) => "Formatting dates is not enabled, enable the \"format_datetime\" feature to do so",
            Formatter::Time(_) => "Formatting time is not enabled, enable the \"format_datetime\" feature to do so",
            Formatter::DateTime(_, _) => "Formatting datetime is not enabled, enable the \"format_datetime\" feature to do so",
            Formatter::List(_, _) => "Formatting lists is not enabled, enable the \"format_list\" feature to do so",
        }
    }
}

fn from_args_helper<'a, T: Default, S: PartialEq + PartialEq<&'a str>>(
    args: Option<&[(S, S)]>,
    name: &'a str,
    f: impl Fn(&S) -> Option<T>,
) -> T {
    let Some(args) = args else {
        return Default::default();
    };
    for (arg_name, value) in args {
        if arg_name != &name {
            continue;
        }
        if let Some(v) = f(value) {
            return v;
        }
    }
    Default::default()
}

macro_rules! impl_from_args {
    ($name:literal, $($arg_name:literal => $value:expr,)*) => {
        pub fn from_args<'a, S: PartialEq + PartialEq<&'a str>>(args: Option<&[(S, S)]>) -> Self {
        from_args_helper(args, $name, |arg| {
            $(
                if arg == &$arg_name {
                    Some($value)
                } else
            )*
            {
                None
            }
        })
    }
    }
}

macro_rules! impl_length {
    ($t:ty, $arg_name:literal, $name:ident) => {
        impl $t {
            impl_from_args! {
                $arg_name,
                "full" => Self::Full,
                "long" => Self::Long,
                "medium" => Self::Medium,
                "short" => Self::Short,
            }
        }
    };
}

impl_length!(DateLength, "date_length", Date);
impl_length!(TimeLength, "time_length", Time);

impl CurrencyCode {
    pub fn from_args<'a, S: PartialEq + PartialEq<&'a str> + ToString>(
        args: Option<&[(S, S)]>,
    ) -> Self {
        from_args_helper(args, "currency_code", |arg| {
            match TinyAsciiStr::from_str(arg.to_string().as_str()) {
                Err(_) => None,
                Ok(code) => Some(Self(code)),
            }
        })
    }
}

impl CurrencyWidth {
    impl_from_args! {
        "width",
        "short" => Self::Short,
        "narrow" => Self::Narrow,
    }
}

impl GroupingStrategy {
    impl_from_args! {
        "grouping_strategy",
        "auto" => Self::Auto,
        "never" => Self::Never,
        "always" => Self::Always,
        "min2" => Self::Min2,
    }
}

impl ListType {
    impl_from_args! {
        "list_type",
        "and" => Self::And,
        "or" => Self::Or,
        "unit" => Self::Unit,
    }
}

impl ListStyle {
    impl_from_args! {
        "list_style",
        "wide" => Self::Wide,
        "short" => Self::Short,
        "narrow" => Self::Narrow,
    }
}
