use lure_cli::Command as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match lure_cli::Cli::parse().subcommand {
        lure_cli::Subcommands::Config(config) => config.run().await.map_err(Into::into),
        lure_cli::Subcommands::Start(start) => start.run().await.map_err(Into::into),
    }
}
