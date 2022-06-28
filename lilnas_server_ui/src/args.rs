#[derive(clap::Parser, Debug)]
#[clap(author)]
pub(crate) struct App {
    #[clap(subcommand)]
    pub(crate) action: Action,
}

#[derive(clap::Subcommand, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    /// Initializes the configuration file.
    Init,
    /// Removes the configuration file.
    Reset,
    /// Adds Logins and Folders.
    Add,
    /// Prints current configurations.
    Info,
}
