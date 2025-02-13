use core::future::Future;
use core::{task::Poll, time::Duration};

use futures::Stream;
use lure_service_common::Service as _;
use reqwest::{ClientBuilder, StatusCode, Url};
use secrecy::ExposeSecret as _;
use tokio::time::{interval, Interval};

pub struct Service {
    http_client: reqwest::Client,
    interval: Interval,
    options: lure_service_lastfm_config::Options,
}

impl Service {
    pub fn try_new(options: lure_service_lastfm_config::Options) -> Result<Self, ServiceError> {
        Ok(Self {
            http_client: ClientBuilder::new().build()?,
            interval: interval(Duration::from_secs(options.check_interval)),
            options,
        })
    }
}

#[async_trait::async_trait]
impl lure_service_common::Service for Service {
    async fn get_current_playing_track(
        &self,
    ) -> Result<Option<lure_service_common::TrackInfo>, anyhow::Error> {
        let url = Url::parse_with_params(
            "https://ws.audioscrobbler.com/2.0/",
            &[
                ("method", "user.getrecenttracks"),
                ("user", &self.options.username),
                ("api_key", self.options.api_key.expose_secret()),
                ("limit", "1"),
                ("format", "json"),
            ],
        )
        .map_err(|error| ServiceError::Anyhow(error.into()))?;

        match self
            .http_client
            .get(url)
            .send()
            .await?
            .handle_user_friendly_error()
            .await
        {
            Ok(response) => {
                let mut recent_tracks: lastfm_models::user::get_recent_tracks::Data =
                    response.json().await?;

                if let Some(track) = recent_tracks.recenttracks.track.first_mut() {
                    if track
                        .attr
                        .as_ref()
                        .is_some_and(|attr| attr.nowplaying.as_ref().is_some_and(|np| *np))
                    {
                        return Ok(Some(lure_service_common::TrackInfo {
                            artist: core::mem::take(&mut track.artist.text),
                            title: core::mem::take(&mut track.name),
                        }));
                    }
                }
            }
            Err(error) => return Err(error.into()),
        }

        Ok(None)
    }
}

impl Stream for Service {
    type Item = Result<Option<lure_service_common::TrackInfo>, anyhow::Error>;

    fn poll_next(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        match self.interval.poll_tick(cx) {
            Poll::Ready(_) => Poll::Ready(Some(futures::executor::block_on(
                self.get_current_playing_track(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

// TODO: Deduplicate this, also in
// `lure-service-listenbrainz`.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error(transparent)]
    APIError(#[from] APIError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum APIError {
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Something went wrong with Last.fm API")]
    OperationFailed,
    #[error("Provided API key is invalid")]
    InvalidAPIKey,
    #[error("API is temporarily offline")]
    ServiceOffline,
    #[error("A temporary error occurred")]
    TemporaryError,
    #[error("API key has been suspended")]
    SuspendedAPIKey,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Unexpected API error: {0}")]
    Unexpected(String),
}

// TODO: Find a way to deduplicate this. Almost
// same code is used in many places.
pub trait HandleServiceAPIError: Sized {
    type Error: core::error::Error;

    fn handle_user_friendly_error(self) -> impl Future<Output = Result<Self, Self::Error>>;
}

impl HandleServiceAPIError for reqwest::Response {
    type Error = ServiceError;

    async fn handle_user_friendly_error(self) -> Result<Self, Self::Error> {
        match self.status() {
            StatusCode::OK => Ok(self),
            StatusCode::FORBIDDEN => {
                let error: lastfm_models::user::get_recent_tracks::Error = self.json().await?;
                match error.error {
                    4 => Err(APIError::AuthenticationFailed.into()),
                    8 => Err(APIError::OperationFailed.into()),
                    10 => Err(APIError::InvalidAPIKey.into()),
                    11 => Err(APIError::ServiceOffline.into()),
                    16 => Err(APIError::TemporaryError.into()),
                    26 => Err(APIError::SuspendedAPIKey.into()),
                    29 => Err(APIError::RateLimitExceeded.into()),
                    _ => Err(APIError::Unexpected(error.message).into()),
                }
            }
            _ => Err(
                APIError::Unexpected(format!("Unexpected status code: {}", self.status())).into(),
            ),
        }
    }
}
