#![allow(unused_macros)]

use std::{
    fs::File,
    io::{BufRead, Write},
    path::PathBuf,
    str::FromStr,
};

use args::Action;
use clap::Parser;
use color_eyre::Result;
use config::{load_application_data, load_config_path, save_application_data, LoginInfo};
use once_cell::sync::OnceCell;
use owo_colors::OwoColorize;
use parking_lot::RwLock;

use crate::{
    args::App,
    config::{initialization_status, ServerConfiguration},
};

mod args;
mod config;
mod macros;

pub(crate) static SERVER_CONFIG: OnceCell<RwLock<ServerConfiguration>> = OnceCell::new();

const SALT: [u8; 16] = [
    0x3b, 0x62, 0x16, 0x1d, 0xfe, 0xb5, 0xab, 0x0e, 0x04, 0x5e, 0x01, 0x96, 0xaf, 0x49, 0x6b, 0x7a,
];
const COST: u32 = 12;

fn setup_application() -> Result<()> {
    log::trace!("Setting Up Application...");

    let initialization_status = initialization_status();
    if initialization_status.done() {
        return Ok(());
    }

    log::trace!("Capturing ServerConfiguration parameters from write lock");
    let mut lock = SERVER_CONFIG.get().unwrap().write();

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

            let info = query_login_info(std::io::stdin().lock(), std::io::stdout())?;
            if !lock.get_logins_mut().insert(info) {
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
        lock.get_folders_mut().insert(path);
    }

    log::trace!("Releasing ServerConfiguration parameters from write lock");
    Ok(())
}

fn query_again(prompt: impl std::fmt::Display) -> Result<bool> {
    let mut data = String::new();
    try_print!("{}", prompt)?;
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut data)?;

    match data.trim_end() {
        "n" | "N" => Ok(false),
        _ => Ok(true),
    }
}

fn add_login() -> Result<bool> {
    log::trace!("Capturing ServerConfiguration parameters from write lock");
    let mut lock = SERVER_CONFIG.get().unwrap().write();
    if !query_again("Add another login? (Y/n) ".yellow())? {
        return Ok(false);
    }
    let info = query_login_info(std::io::stdin().lock(), std::io::stdout())?;
    if !lock.get_logins_mut().insert(info) {
        log::error!("Could not add duplicate user. Please try again.");
    }
    log::trace!("Releasing ServerConfiguration parameters from write lock");
    Ok(true)
}

fn add_folder() -> Result<bool> {
    log::trace!("Capturing ServerConfiguration parameters from write lock");
    let mut lock = SERVER_CONFIG.get().unwrap().write();

    if !query_again("Add another folder? (Y/n) ".yellow())? {
        return Ok(false);
    }
    let path = query_folder_info(std::io::stdin().lock(), std::io::stdout())?;
    lock.get_folders_mut().insert(path);
    log::trace!("Releasing ServerConfiguration parameters from write lock");
    Ok(true)
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

fn query_login_info(mut stdin: impl BufRead, mut stdout: impl Write) -> Result<LoginInfo> {
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
            writeln!(stdout, "{}", "Usernames do not match!".red())?;
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

        break Ok(LoginInfo::new(
            username,
            &bcrypt::bcrypt(COST, SALT, password.as_bytes()),
        ));
    }
}

fn main() -> Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .env()
        .init()
        .unwrap();

    let args = App::try_parse()?;
    let config_path = load_config_path()?;
    match load_application_data(&config_path) {
        e @ Err(_) if args.action == Action::Reset => {
            File::options()
                .write(true)
                .truncate(true)
                .open(&config_path)?;
            e?
        }
        e => e?,
    };

    match args.action {
        Action::Init => {
            setup_application()?;
            log::debug!("Completed application setup");
        }
        Action::Reset => {
            let mut lock = SERVER_CONFIG.get().unwrap().write();
            lock.get_folders_mut().clear();
            lock.get_logins_mut().clear();
            log::debug!("Completed resetting configuration");
        }
        Action::Add => {
            if !initialization_status().done() {
                try_eprintln!(
                    "The program has not been initialized. Please run the `init` subcommand first."
                )?;
                std::process::exit(1);
            }
            while add_login()? {}
            while add_folder()? {}
            log::debug!("Completed adding logins and folders");
        }
        Action::Info => {
            if !initialization_status().done() {
                try_eprintln!(
                    "The program has not been initialized. Please run the `init` subcommand first."
                )?;
                std::process::exit(1);
            }
            try_println!("{:#?}", &*SERVER_CONFIG.get().unwrap().read())?;
            log::debug!("Completed printing current configuration");
        }
    }

    save_application_data(config_path)?;
    // TODO: update_daemon()?;
    Ok(())
}
