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
    Date(DateTimeLength, DateTimeAlignment, DateTimeYearStyle),
    Time(DateTimeLength, DateTimeAlignment, DateTimeTimePrecision),
    DateTime(
        DateTimeLength,
        DateTimeAlignment,
        DateTimeTimePrecision,
        DateTimeYearStyle,
    ),
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
pub enum DateTimeLength {
    Long,
    #[default]
    Medium,
    Short,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateTimeAlignment {
    #[default]
    Auto,
    Column,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateTimeTimePrecision {
    Hour,
    Minute,
    #[default]
    Second,
    Subsecond(DateTimeSubsecondDigits),
    MinuteOptional,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateTimeSubsecondDigits {
    S1 = 1,
    S2 = 2,
    #[default]
    S3 = 3,
    S4 = 4,
    S5 = 5,
    S6 = 6,
    S7 = 7,
    S8 = 8,
    S9 = 9,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateTimeYearStyle {
    #[default]
    Auto,
    Full,
    WithEra,
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
            let formatter = Formatter::DateTime(
                DateTimeLength::from_args(args),
                DateTimeAlignment::from_args(args),
                DateTimeTimePrecision::from_args(args),
                DateTimeYearStyle::from_args(args),
            );
            if cfg!(feature = "format_datetime") || SKIP_ICU_CFG.get() {
                Ok(Some(formatter))
            } else {
                Err(formatter)
            }
        } else if name == "date" {
            let formatter = Formatter::Date(
                DateTimeLength::from_args(args),
                DateTimeAlignment::from_args(args),
                DateTimeYearStyle::from_args(args),
            );
            if cfg!(feature = "format_datetime") || SKIP_ICU_CFG.get() {
                Ok(Some(formatter))
            } else {
                Err(formatter)
            }
        } else if name == "time" {
            let formatter = Formatter::Time(
                DateTimeLength::from_args(args),
                DateTimeAlignment::from_args(args),
                DateTimeTimePrecision::from_args(args),
            );
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
            Formatter::Date(_, _, _) => "Formatting dates is not enabled, enable the \"format_datetime\" feature to do so",
            Formatter::Time(_, _, _) => "Formatting time is not enabled, enable the \"format_datetime\" feature to do so",
            Formatter::DateTime(_, _, _, _) => "Formatting datetime is not enabled, enable the \"format_datetime\" feature to do so",
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

impl DateTimeLength {
    impl_from_args! {
        "length",
        "long" => Self::Long,
        "medium" => Self::Medium,
        "short" => Self::Short,
    }
}

impl DateTimeAlignment {
    impl_from_args! {
        "alignment",
        "auto" => Self::Auto,
        "column" => Self::Column,
    }
}

impl DateTimeYearStyle {
    impl_from_args! {
        "alignment",
        "auto" => Self::Auto,
        "full" => Self::Full,
        "with_era" => Self::WithEra,
    }
}

impl DateTimeTimePrecision {
    impl_from_args! {
        "time_precision",
        "hour" => Self::Hour,
        "minute" => Self::Minute,
        "second" => Self::Second,
        "subsecond_s1" => Self::Subsecond(DateTimeSubsecondDigits::S1),
        "subsecond_s2" => Self::Subsecond(DateTimeSubsecondDigits::S2),
        "subsecond_s3" => Self::Subsecond(DateTimeSubsecondDigits::S3),
        "subsecond_s4" => Self::Subsecond(DateTimeSubsecondDigits::S4),
        "subsecond_s5" => Self::Subsecond(DateTimeSubsecondDigits::S5),
        "subsecond_s6" => Self::Subsecond(DateTimeSubsecondDigits::S6),
        "subsecond_s7" => Self::Subsecond(DateTimeSubsecondDigits::S7),
        "subsecond_s8" => Self::Subsecond(DateTimeSubsecondDigits::S8),
        "subsecond_s9" => Self::Subsecond(DateTimeSubsecondDigits::S9),
        "minute_optional" => Self::MinuteOptional,
    }
}
impl CurrencyCode {
    pub fn from_args<'a, S: PartialEq + PartialEq<&'a str> + ToString>(
        args: Option<&[(S, S)]>,
    ) -> Self {
        from_args_helper(
            args,
            "currency_code",
            |arg| match TinyAsciiStr::try_from_str(arg.to_string().as_str()) {
                Err(_) => None,
                Ok(code) => Some(Self(code)),
            },
        )
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
