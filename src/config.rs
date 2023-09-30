use confique::{toml, Config};
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Config)]
pub struct Options {
    /// Which platform to check current listening from
    #[config(env = "LURE_PLATFORM")]
    pub platform: String,

    /// Revolt session token to set the status
    #[config(env = "LURE_REVOLT_SESSION_TOKEN")]
    pub session_token: String,

    /// Revolt status options to set
    #[config(nested)]
    pub status: StatusOptions,

    /// Last.fm platform specific options
    /// Can be skipped if this platform is not being used.
    #[cfg(feature = "lastfm")]
    #[config(nested)]
    pub lastfm: LastFMOptions,

    /// ListenBrainz platform specific options
    /// Can be skipped if this platform is not being used
    #[cfg(feature = "listenbrainz")]
    #[config(nested)]
    pub listenbrainz: ListenBrainzOptions,
}

impl Options {
    pub fn generate_config() -> String {
        toml::template::<Self>(&confique::toml::FormatOptions::default())
    }

    pub async fn create_config() -> anyhow::Result<()> {
        let config = Self::generate_config();
        let path = dirs::config_local_dir()
            .expect("unsupported operating system or platform")
            .join("lure")
            .join("config")
            .with_extension("toml");

        tokio::fs::create_dir_all(&path.parent().unwrap()).await?;

        if path.try_exists()? {
            return Err(anyhow::anyhow!("configuration file already exists."));
        }

        let mut file = File::create(&path).await?;
        file.write_all(config.as_bytes()).await?;

        tracing::info!("created a configuration file at `{}`", path.display());

        Ok(())
    }
}

#[derive(Config)]
pub struct StatusOptions {
    /// Status template to use when setting the status
    #[config(default = "🎵 %ARTIST% – %NAME%", env = "LURE_STATUS_TEMPLATE")]
    pub template: String,

    /// Idle status message to use when not listening anything
    #[config(env = "LURE_STATUS_IDLE")]
    pub idle: Option<String>,
}

#[cfg(feature = "lastfm")]
#[derive(Config)]
pub struct LastFMOptions {
    /// Last.fm username to check current listening status from
    #[config(env = "LURE_LASTFM_USER")]
    pub user: Option<String>,

    /// Last.fm API key to be able to check current listening through API
    #[config(env = "LURE_LASTFM_API_KEY")]
    pub api_key: Option<String>,

    /// Check interval
    #[config(default = 12, env = "LURE_LASTFM_CHECK_INTERVAL")]
    pub check_interval: u64,
}

#[cfg(feature = "listenbrainz")]
#[derive(Config)]
pub struct ListenBrainzOptions {
    /// ListenBrainz username to check current listening status from
    #[config(env = "LURE_LISTENBRAINZ_USER")]
    pub user: Option<String>,

    /// ListenBrainz API URL to send the API requests to
    #[config(
        default = "https://api.listenbrainz.org",
        env = "LURE_LISTENBRAINZ_API_URL"
    )]
    pub api_url: String,

    /// Check interval
    #[config(default = 12, env = "LURE_LISTENBRAINZ_CHECK_INTERVAL")]
    pub check_interval: u64,
}
