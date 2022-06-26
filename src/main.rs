#![allow(unused_macros)]

use std::{
    collections::BTreeSet,
    fs::{create_dir_all, File},
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
    str::FromStr,
};

use args::Action;
use clap::Parser;
use color_eyre::Result;
use once_cell::sync::OnceCell;
use owo_colors::OwoColorize;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::args::App;

mod args;
mod macros;

#[derive(Serialize, Deserialize, Debug)]
struct LoginInfo {
    user: String,
    base64_passwd_hash: String,
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
struct ServerConfiguration {
    folders: BTreeSet<PathBuf>,
    login_info: BTreeSet<LoginInfo>,
}

static SERVER_CONFIG: OnceCell<RwLock<ServerConfiguration>> = OnceCell::new();
const SALT: [u8; 16] = [
    0x3b, 0x62, 0x16, 0x1d, 0xfe, 0xb5, 0xab, 0x0e, 0x04, 0x5e, 0x01, 0x96, 0xaf, 0x49, 0x6b, 0x7a,
];
const COST: u32 = 12;

fn load_config_dir() -> Result<PathBuf> {
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

    match config_dir {
        Some(mut config_dir) if config_dir.is_dir() => {
            config_dir.push("lilnasxium");
            if !config_dir.is_dir() {
                create_dir_all(&config_dir)?;
            }
            load_application_data(config_dir.clone())?;
            assert!(
                SERVER_CONFIG.get().is_some(),
                "SERVER_CONFIG was None when expected to be Some(_)"
            );

            Ok(config_dir)
        }
        _ => {
            try_println!("Could not set up application configuration folder.")?;
            std::process::exit(1);
        }
    }
}

fn setup_application() -> Result<()> {
    log::trace!("Setting Up Application...");

    let initialization_status = initialization_status();
    if initialization_status.done() {
        return Ok(());
    }

    log::trace!("Capturing ServerConfiguration parameters from write lock");
    let mut lock = SERVER_CONFIG.get().unwrap().write();
    let ServerConfiguration {
        ref mut login_info,
        ref mut folders,
    } = *lock;

    if !initialization_status.login_info {
        loop {
            let mut data = String::new();
            try_print!(
                "{}",
                "No login has been set up. Would you like to set it up now? (Y/n) ".yellow()
            )?;
            std::io::stdout().flush()?;
            std::io::stdin().read_line(&mut data)?;

            match data.trim_end() {
                "n" | "N" => {
                    std::process::exit(1);
                }
                _ => {}
            }

            let (uname, hashed_passwd) =
                query_login_info(std::io::stdin().lock(), std::io::stdout())?;
            if !login_info.insert(LoginInfo {
                user: uname,
                base64_passwd_hash: hashed_passwd,
            }) {
                log::error!("Cannot add duplicate login. Try adding a different login.");
            } else {
                break;
            }
        }
    }

    if !initialization_status.folders {
        let mut data = String::new();
        try_print!(
            "{}",
            "No folders have been added. Would you like to add one now? (Y/n) ".yellow()
        )?;
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut data)?;

        match data.trim_end() {
            "n" | "N" => std::process::exit(1),
            _ => {}
        }
        let path = query_folder_info(std::io::stdin().lock(), std::io::stdout())?;
        folders.insert(path);
    }

    log::trace!("Releasing ServerConfiguration parameters from write lock");
    Ok(())
}

fn initialization_status() -> InitializationStatus {
    let ServerConfiguration {
        folders,
        login_info,
    } = &*SERVER_CONFIG.get().unwrap().read();

    InitializationStatus {
        folders: !folders.is_empty(),
        login_info: !login_info.is_empty(),
    }
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) struct InitializationStatus {
    pub(crate) folders: bool,
    pub(crate) login_info: bool,
}

impl InitializationStatus {
    fn done(&self) -> bool {
        self.folders && self.login_info
    }
}

fn add_login() -> Result<()> {
    log::trace!("Capturing ServerConfiguration parameters from write lock");
    let mut lock = SERVER_CONFIG.get().unwrap().write();
    let ServerConfiguration {
        ref mut login_info, ..
    } = *lock;
    loop {
        let mut data = String::new();
        try_print!("{}", "Add another login? (Y/n) ".yellow())?;
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut data)?;

        match data.trim_end() {
            "n" | "N" => return Ok(()),
            _ => {}
        }

        let (uname, hashed_passwd) = query_login_info(std::io::stdin().lock(), std::io::stdout())?;
        if !login_info.insert(LoginInfo {
            user: uname,
            base64_passwd_hash: hashed_passwd,
        }) {
            log::error!("Could not add duplicate user. Please try again.");
        } else {
            break;
        }
    }

    log::trace!("Releasing ServerConfiguration parameters from write lock");
    Ok(())
}

fn add_folder() -> Result<()> {
    log::trace!("Capturing ServerConfiguration parameters from write lock");
    let mut lock = SERVER_CONFIG.get().unwrap().write();
    let ServerConfiguration {
        ref mut folders, ..
    } = *lock;
    let mut data = String::new();
    try_print!("{}", "Add another folder? (Y/n) ".yellow())?;
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut data)?;

    match data.trim_end() {
        "n" | "N" => return Ok(()),
        _ => {}
    }
    let path = query_folder_info(std::io::stdin().lock(), std::io::stdout())?;
    folders.insert(path);
    log::trace!("Releasing ServerConfiguration parameters from write lock");
    Ok(())
}

fn save_application_data(mut config_dir: PathBuf) -> Result<()> {
    log::trace!("Saving Application Data...");
    config_dir.push("config.toml");

    let config: &ServerConfiguration = &*SERVER_CONFIG.get().unwrap().read();
    log::trace!("Serializing configuration data...\n{:#?}", config);
    let data = &toml::to_vec(config)?;

    log::trace!("Writing configuration to file...");
    File::options()
        .write(true)
        .open(&mut config_dir)?
        .write_all(data)?;

    log::debug!(
        "Wrote configuration to {}",
        config_dir.to_string_lossy().yellow()
    );

    Ok(())
}

fn query_folder_info(mut stdin: impl BufRead, mut stdout: impl Write) -> Result<PathBuf> {
    log::trace!("Querying Folder Info...");

    loop {
        let mut data = String::new();

        write!(stdout, "{}", "ABSOLUTE path to folder: ".green())?;
        stdout.flush()?;
        stdin.read_line(&mut data)?;
        data.pop();

        match PathBuf::from_str(&data) {
            Ok(path) if path.is_dir() && path.is_absolute() => break Ok(path),
            _ => writeln!(stdout, "{}", "Invalid path, please try again.".red())?,
        }
    }
}

fn query_login_info(mut stdin: impl BufRead, mut stdout: impl Write) -> Result<(String, String)> {
    log::trace!("Querying Login Info...");

    loop {
        let mut username = String::new();
        let mut password;

        loop {
            write!(stdout, "{}", "Username: ".green())?;
            stdout.flush()?;
            stdin.read_line(&mut username)?;
            username.pop();

            if username.len() >= 4 && username.len() <= 20 {
                break;
            } else {
                try_println!(
                    "Your username must be between 4 and 20 (inclusive) characters long."
                )?;
            }
        }

        loop {
            #[cfg(test)]
            {
                password = rpassword::prompt_password_from_bufread(
                    &mut stdin,
                    &mut stdout,
                    "Password: ".green(),
                )?;
            }
            #[cfg(not(test))]
            {
                password = rpassword::prompt_password("Password: ".green())?;
            }

            if password.len() >= 12 && password.len() <= 72 {
                break;
            } else {
                try_println!(
                    "Your password must be between 12 and 72 (inclusive) characters long."
                )?;
            }
        }

        let mut confirmed_username = String::new();
        write!(stdout, "{}", "Confirm Username: ".green())?;
        stdout.flush()?;
        stdin.read_line(&mut confirmed_username)?;
        confirmed_username.pop();

        if confirmed_username != username {
            password.clear();
            writeln!(stdout, "{}", "Passwords do not match!".red())?;
            continue;
        }

        #[cfg(test)]
        let mut confirmed_password = rpassword::prompt_password_from_bufread(
            &mut stdin,
            &mut stdout,
            "Confirm Password: ".green(),
        )?;
        #[cfg(not(test))]
        let mut confirmed_password = rpassword::prompt_password("Confirm Password: ".green())?;

        if confirmed_password != password {
            password.clear();
            confirmed_password.clear();
            writeln!(stdout, "{}", "Passwords do not match!".red())?;
            continue;
        }

        break Ok((
            username,
            base64::encode(bcrypt::bcrypt(COST, SALT, password.as_bytes())),
        ));
    }
}

fn load_application_data(mut dir: PathBuf) -> Result<()> {
    dir.push("config.toml");

    let file = if dir.is_file() {
        File::options().read(true).open(dir)?
    } else {
        File::create(&dir)?;
        File::options().read(true).open(dir)?
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

    Ok(())
}

fn main() -> Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .env()
        .init()
        .unwrap();

    let args = App::try_parse()?;
    let config_dir = load_config_dir()?;

    match args.action {
        Action::Init => {
            setup_application()?;
            log::debug!("Completed application setup");
        }
        Action::Add => {
            if !initialization_status().done() {
                try_eprintln!(
                    "The program has not been initialized. Please run the `init` subcommand first."
                )?;
                std::process::exit(1);
            }
            add_login()?;
            add_folder()?;
        }
    }

    save_application_data(config_dir)?;
    // TODO: update_daemon()?;
    Ok(())
}
