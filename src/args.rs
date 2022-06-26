#[derive(clap::Parser, Debug)]
#[clap(author)]
pub(crate) struct App {
    #[clap(subcommand)]
    pub(crate) action: Action
}

#[derive(clap::Subcommand, Debug)]
pub(crate) enum Action {
    /// Initializes the configuration file.
    Init,
    /// Adds Logins and Folders.
    Add,
}
