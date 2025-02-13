use futures::Stream;

#[derive(Debug, PartialEq, Eq)]
pub struct TrackInfo {
    pub artist: String,
    pub title: String,
}

#[async_trait::async_trait]
pub trait Service: Stream<Item = Result<Option<TrackInfo>, anyhow::Error>> + Send + Sync {
    async fn get_current_playing_track(&self) -> Result<Option<TrackInfo>, anyhow::Error>;
}

pub trait ServiceCustomError: core::error::Error + Send + Sync + 'static {}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError<T: ServiceCustomError> {
    #[error(transparent)]
    CustomError(T),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl<T: ServiceCustomError> From<T> for ServiceError<T> {
    fn from(error: T) -> Self {
        ServiceError::CustomError(error)
    }
}
