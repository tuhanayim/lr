pub mod user {
    pub mod playing_now {
        #[derive(Debug, serde::Deserialize)]
        pub struct Data {
            pub payload: Payload,
        }

        #[derive(Debug, serde::Deserialize)]
        pub struct Payload {
            pub listens: Vec<Listen>,
        }

        #[derive(Debug, serde::Deserialize)]
        pub struct Listen {
            pub playing_now: bool,
            pub track_metadata: TrackMetadata,
        }

        #[derive(Debug, serde::Deserialize)]
        pub struct TrackMetadata {
            pub artist_name: String,
            pub track_name: String,
        }
    }
}
