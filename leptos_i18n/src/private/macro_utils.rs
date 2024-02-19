use std::{borrow::Cow, ops::Deref};

pub trait BuildStr: Sized {
    #[inline]
    fn build(self) -> Self {
        self
    }

    #[inline]
    fn build_display(self) -> Self {
        self
    }

    fn build_string(self) -> Cow<'static, str>;
}

impl BuildStr for &'static str {
    #[inline]
    fn build_string(self) -> Cow<'static, str> {
        Cow::Borrowed(self)
    }
}

#[repr(transparent)]
pub struct SizedString<const N: usize>([u8; N]);

impl<const N: usize> SizedString<N> {
    pub const fn try_new(s: &str) -> Option<Self> {
        if s.len() != N {
            return None;
        }
        // SAFETY:
        // `s` is exactly N bytes in len, so casting it to a `[u8; N]` is totally valid.
        // There is way to do this without unsafe, with for exemple `TryInto<&[u8; N]>` for &[u8],
        // or create a buffer and manually filling it, but none of these methods are const,
        // and it makes things easier if this method can be const.
        let bytes = s.as_bytes().as_ptr().cast::<[u8; N]>();
        Some(SizedString(unsafe { *bytes }))
    }

    #[track_caller]
    pub const fn new(s: &str) -> Self {
        #[cold]
        #[track_caller]
        #[inline(never)]
        const fn empty() -> ! {
            panic!("Receive &str of wrong len.");
        }
        match Self::try_new(s) {
            Some(v) => v,
            None => empty(),
        }
    }
}

impl<const N: usize> Deref for SizedString<N> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // only way to create this type is through a valid str,
        // so the internal buffer is a valid str.
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

pub trait ParseTranslation: Sized {
    fn parse(buff: &mut &str) -> Option<Self>;
    fn pop_str<'a>(buff: &mut &'a str, size: usize) -> Option<&'a str> {
        let (s, rest) = Self::split_str(buff, size)?;
        *buff = rest;
        Some(s)
    }
    fn split_str(s: &str, at: usize) -> Option<(&str, &str)> {
        // this is a replica of `str::split_at` but doesn't panic
        // SAFETY:
        // The len is checked inside `is_char_boundary` and it is safe to split at a char boundary.
        s.is_char_boundary(at)
            .then(|| unsafe { (s.get_unchecked(..at), s.get_unchecked(at..)) })
    }
}

impl<const N: usize> ParseTranslation for SizedString<N> {
    fn parse(buff: &mut &str) -> Option<Self> {
        let s = Self::pop_str(buff, N)?;
        Self::try_new(s)
    }
}
