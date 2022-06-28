use std::{
    collections::BTreeSet,
    fs::{create_dir_all, File},
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
};

use color_eyre::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::SERVER_CONFIG;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct LoginInfo {
    user: String,
    base64_passwd_hash: String,
}

impl LoginInfo {
    pub(crate) fn new(username: String, passwd_hash: &[u8]) -> Self {
        Self {
            user: username,
            base64_passwd_hash: base64::encode(passwd_hash),
        }
    }
}

impl PartialEq for LoginInfo {
    fn eq(&self, other: &Self) -> bool {
        self.user.eq(&other.user)
    }
}

impl PartialOrd for LoginInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.user.partial_cmp(&other.user)
    }
}

impl Eq for LoginInfo {}
impl Ord for LoginInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.user.cmp(&other.user)
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub(crate) struct ServerConfiguration {
    folders: BTreeSet<PathBuf>,
    logins: BTreeSet<LoginInfo>,
}

impl ServerConfiguration {
    pub(crate) fn get_folders_mut(&mut self) -> &mut BTreeSet<PathBuf> {
        &mut self.folders
    }
    pub(crate) fn get_logins_mut(&mut self) -> &mut BTreeSet<LoginInfo> {
        &mut self.logins
    }
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) struct InitializationStatus {
    pub(crate) folders: bool,
    pub(crate) login_info: bool,
}

impl InitializationStatus {
    pub(crate) fn done(&self) -> bool {
        self.folders && self.login_info
    }
}

pub(crate) fn initialization_status() -> InitializationStatus {
    let ServerConfiguration {
        folders,
        logins: login_info,
    } = &*SERVER_CONFIG.get().unwrap().read();

    InitializationStatus {
        folders: !folders.is_empty(),
        login_info: !login_info.is_empty(),
    }
}

pub(crate) fn load_config_path() -> Result<PathBuf> {
    let config_dir = if !cfg!(test) {
        dirs::config_dir()
    } else {
        let mut rng = rand::thread_rng();
        let folder_name = std::iter::repeat(())
            .map(|()| rand::Rng::sample(&mut rng, rand::distributions::Alphanumeric))
            .map(char::from)
            .take(16)
            .collect::<String>();
        use std::env::temp_dir;
        Some(temp_dir().join(folder_name))
    };

    config_dir
        .ok_or(color_eyre::eyre::eyre!(
            "Could not set up application configuration folder."
        ))
        .and_then(|mut path| {
            path.push("lilnasxium");
            if !path.is_dir() {
                create_dir_all(&path)?;
            }
            path.push("config.toml");
            Ok(path)
        })
}

pub(crate) fn load_application_data(path: &Path) -> Result<()> {
    let file = if path.is_file() {
        File::options().read(true).open(path)?
    } else {
        File::create(&path)?;
        File::options().read(true).open(path)?
    };

    let contents = {
        let mut data = String::new();
        let mut buf_read = BufReader::new(file);
        buf_read.read_to_string(&mut data)?;

        data
    };

    SERVER_CONFIG.get_or_try_init(|| -> Result<RwLock<ServerConfiguration>> {
        let data = if !contents.is_empty() {
            toml::from_str::<ServerConfiguration>(&contents)?
        } else {
            Default::default()
        };

        Ok(RwLock::new(data))
    })?;

    assert!(
        SERVER_CONFIG.get().is_some(),
        "SERVER_CONFIG was None when expected to be Some(_)"
    );

    Ok(())
}

pub(crate) fn save_application_data(mut config_path: PathBuf) -> Result<()> {
    log::trace!("Saving Application Data...");

    let config: &ServerConfiguration = &*SERVER_CONFIG.get().unwrap().read();
    log::trace!("Serializing configuration data...\n{:#?}", config);
    let data = &toml::to_vec(config)?;

    log::trace!("Writing configuration to file...");
    File::options()
        .write(true)
        .truncate(true)
        .open(&mut config_path)?
        .write_all(data)?;

    log::debug!("Wrote configuration to {}", config_path.display());

    Ok(())
}
