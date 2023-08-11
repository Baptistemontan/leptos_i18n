use crate::{
    error::{Error, Result},
    key::Key,
};
use std::{fs::File, path::Path};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct RawConfigFile {
    pub default: String,
    pub locales: Vec<String>,
}

impl RawConfigFile {
    pub fn new<T: AsRef<Path>>(path: Option<T>) -> Result<RawConfigFile> {
        let path = path
            .as_ref()
            .map(|path| path.as_ref())
            .unwrap_or("./i18n.json".as_ref());
        let cfg_file = File::open(path).map_err(Error::ConfigFileNotFound)?;

        let cfg: RawConfigFile =
            serde_json::from_reader(cfg_file).map_err(Error::ConfigFileDeser)?;

        if cfg.locales.contains(&cfg.default) {
            Ok(cfg)
        } else {
            Err(Error::ConfigFileDefaultMissing(cfg))
        }
    }
}

pub struct ConfigFile<'a> {
    pub default: Key<'a>,
    pub locales: Vec<Key<'a>>,
}

impl<'a> ConfigFile<'a> {
    pub fn new(raw_cfg_file: &'a RawConfigFile) -> Result<Self> {
        let default = Key::new(&raw_cfg_file.default, crate::key::KeyKind::LocaleName)?;
        let locales = raw_cfg_file
            .locales
            .iter()
            .map(|locale| Key::new(locale, crate::key::KeyKind::LocaleName))
            .collect::<Result<_>>()?;

        Ok(ConfigFile { default, locales })
    }
}
