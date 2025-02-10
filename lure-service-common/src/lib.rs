use futures::Stream;

#[derive(Debug, PartialEq, Eq)]
pub struct TrackInfo {
    pub artist: String,
    pub title: String,
}

pub type ServiceError = Box<dyn core::error::Error + Send + Sync>;

#[async_trait::async_trait]
pub trait Service: Stream<Item = Result<Option<TrackInfo>, ServiceError>> + Send + Sync {
    async fn get_current_playing_track(&self) -> Result<Option<TrackInfo>, ServiceError>;
}
