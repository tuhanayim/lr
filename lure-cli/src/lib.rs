// TODO: Maybe move this crate to `lure` directly
// instead?

use core::future::Future;

pub mod config;
pub mod start;

pub trait Command: Sized {
    type Error;

    fn run(&self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

#[derive(Debug, clap::Parser)]
#[command(about, version)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Subcommands,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommands {
    /// Start lure.
    Start(start::Arguments),
    /// Lure configuration options
    #[command(subcommand)]
    Config(config::Subcommands),
}

impl Cli {
    #[must_use]
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
}
