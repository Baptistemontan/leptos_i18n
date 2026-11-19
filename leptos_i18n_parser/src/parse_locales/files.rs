use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    parse_locales::{
        error::{Diagnostics, Error, Result, Warning},
        options::{Config, FileFormat, ParseOptions},
    },
    utils::Key,
};

#[derive(Debug, Clone)]
pub enum TranslationFiles<'a> {
    NameSpaces {
        namespaces: HashMap<&'a str, LocaleFiles<'a>>,
    },
    Locales {
        locales: LocaleFiles<'a>,
    },
}

#[derive(Debug, Clone)]
pub struct LocaleFiles<'a> {
    pub files: HashMap<&'a str, (Option<PathBuf>, Cow<'a, str>)>,
}

fn read_file(path: &mut PathBuf, file_format: &FileFormat) -> Result<String> {
    let mut errs = vec![];

    for ext in file_format.get_files_exts() {
        path.set_extension(ext);
        #[allow(clippy::needless_borrows_for_generic_args)]
        // see https://github.com/rust-lang/rust-clippy/issues/12856
        match fs::read_to_string(&path) {
            Ok(file) => return Ok(file),
            Err(err) => {
                errs.push((path.to_owned(), err));
            }
        };
    }

    Err(Error::LocaleFileNotFound(errs).into())
}

fn track_file(
    tracked_files: &mut Vec<String>,
    locale: &Key,
    namespace: Option<&Key>,
    path: &Path,
    diag: &Diagnostics,
) {
    if let Some(path) = path.as_os_str().to_str().map(ToOwned::to_owned) {
        tracked_files.push(path);
    } else {
        diag.emit_warning(Warning::NonUnicodePath {
            locale: locale.clone(),
            namespace: namespace.cloned(),
            path: path.to_owned(),
        });
    }
}

fn load_namespaces_files<'a>(
    locales_dir_path: &mut PathBuf,
    namespaces: &'a [Key],
    locale_keys: &'a [Key],
    diag: &Diagnostics,
    tracked_files: &mut Vec<String>,
    options: &ParseOptions,
) -> Result<TranslationFiles<'a>> {
    let mut namespaces_files = HashMap::with_capacity(namespaces.len());
    for ns in namespaces {
        let locales = load_locale_files(
            locales_dir_path,
            locale_keys,
            Some(ns),
            diag,
            tracked_files,
            options,
        )?;
        namespaces_files.insert(&*ns.name, locales);
    }
    Ok(TranslationFiles::NameSpaces {
        namespaces: namespaces_files,
    })
}

fn load_locale_files<'a>(
    locales_dir_path: &mut PathBuf,
    locale_keys: &'a [Key],
    namespace: Option<&'a Key>,
    diag: &Diagnostics,
    tracked_files: &mut Vec<String>,
    options: &ParseOptions,
) -> Result<LocaleFiles<'a>> {
    let mut files = HashMap::with_capacity(locale_keys.len());
    let ns_path = namespace.map::<&Path, _>(|ns| ns.name.as_ref().as_ref());
    for locale in locale_keys {
        locales_dir_path.push(&*locale.name);
        if let Some(ns_path) = ns_path {
            locales_dir_path.push(ns_path);
        }

        let locale_file = read_file(locales_dir_path, &options.file_format)?;
        files.insert(
            &*locale.name,
            (Some(locales_dir_path.clone()), Cow::Owned(locale_file)),
        );

        track_file(tracked_files, locale, namespace, locales_dir_path, diag);

        if ns_path.is_some() {
            locales_dir_path.pop();
        }
        locales_dir_path.pop();
    }
    Ok(LocaleFiles { files })
}

impl<'a> TranslationFiles<'a> {
    pub fn load_files(
        manifest_dir_path: &mut PathBuf,
        diag: &Diagnostics,
        tracked_files: &mut Vec<String>,
        cfg: &'a Config,
    ) -> Result<Self> {
        manifest_dir_path.push(&cfg.locales_path);
        if !cfg.namespaces.is_empty() {
            load_namespaces_files(
                manifest_dir_path,
                &cfg.namespaces,
                &cfg.locales,
                diag,
                tracked_files,
                &cfg.options,
            )
        } else {
            let locale_files = load_locale_files(
                manifest_dir_path,
                &cfg.locales,
                None,
                diag,
                tracked_files,
                &cfg.options,
            )?;
            Ok(TranslationFiles::Locales {
                locales: locale_files,
            })
        }
    }
}
