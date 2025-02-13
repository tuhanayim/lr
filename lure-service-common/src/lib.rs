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
