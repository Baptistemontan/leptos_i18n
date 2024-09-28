#![doc(hidden)]

use crate::Locale;

#[cfg(feature = "dynamic_load")]
pub use async_once_cell::OnceCell;

pub trait TranslationUnit {
    type Locale: Locale;
    const ID: <Self::Locale as Locale>::TranslationUnitId;
    const LOCALE: Self::Locale;
    type Strings: StringArray;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    const STRINGS: Self::Strings;
    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn get_strings_lock() -> &'static OnceCell<Self::Strings>;
    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn request_strings() -> impl std::future::Future<Output = Self::Strings> + Send + Sync + 'static
    {
        let string_lock = Self::get_strings_lock();
        async move {
            let inner = string_lock
                .get_or_init(async {
                    let translations = Locale::request_translations(Self::LOCALE, Self::ID)
                        .await
                        .unwrap();
                    let leaked_string: Self::Strings = StringArray::leak(translations.0);
                    leaked_string
                })
                .await;
            *inner
        }
    }
}

pub trait StringArray: Copy + 'static + Send + Sync {
    fn leak(strings: Vec<String>) -> Self;
    fn as_slice(self) -> &'static [&'static str];
}

impl<const SIZE: usize> StringArray for &'static [&'static str; SIZE] {
    fn leak(strings: Vec<String>) -> Self {
        fn cast_ref(r: &mut str) -> &str {
            r
        }
        let values = strings
            .into_iter()
            .map(String::leak)
            .map(cast_ref)
            .collect::<Box<[&'static str]>>();

        let sized_box: Box<[&'static str; SIZE]> = Box::try_into(values).unwrap();

        Box::leak(sized_box)
    }

    fn as_slice(self) -> &'static [&'static str] {
        self
    }
}

#[cfg(all(feature = "dynamic_load", feature = "ssr"))]
pub type LocaleServerFnOutput = LocaleServerFnOutputServer;

#[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
pub type LocaleServerFnOutput = LocaleServerFnOutputClient;

pub struct LocaleServerFnOutputServer(&'static [&'static str]);
pub struct LocaleServerFnOutputClient(pub Vec<String>);

impl LocaleServerFnOutputServer {
    pub const fn new(strings: &'static [&'static str]) -> Self {
        LocaleServerFnOutputServer(strings)
    }
}

impl LocaleServerFnOutputClient {
    pub fn new(_: &'static [&'static str]) -> Self {
        unreachable!("This function should not have been called on the server !")
    }
}

impl serde::Serialize for LocaleServerFnOutputServer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(self.0, serializer)
    }
}

impl<'de> serde::Deserialize<'de> for LocaleServerFnOutputServer {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unreachable!("This function should not have been called on the server !")
    }
}

impl serde::Serialize for LocaleServerFnOutputClient {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        unreachable!("This function should not have been called on the client !")
    }
}

impl<'de> serde::Deserialize<'de> for LocaleServerFnOutputClient {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arr = serde::Deserialize::deserialize(deserializer)?;
        Ok(LocaleServerFnOutputClient(arr))
    }
}
