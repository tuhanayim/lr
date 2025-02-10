use serde::Deserialize as _;

pub mod user {
    pub mod get_recent_tracks {
        #[derive(Debug, serde::Deserialize)]
        pub struct Data {
            pub recenttracks: RecentTracks,
        }

        #[derive(Debug, serde::Deserialize)]
        pub struct Error {
            pub message: String,
            pub error: u64,
        }

        #[derive(Debug, serde::Deserialize)]
        pub struct RecentTracks {
            pub track: Vec<Track>,
        }

        #[derive(Debug, serde::Deserialize)]
        pub struct Track {
            pub artist: Artist,
            pub name: String,
            #[serde(rename = "@attr")]
            pub attr: Option<TrackAttr>,
        }

        #[derive(Debug, serde::Deserialize)]
        pub struct Artist {
            #[serde(rename = "#text")]
            pub text: String,
        }

        #[derive(Debug, serde::Deserialize)]
        pub struct TrackAttr {
            #[serde(deserialize_with = "crate::bool_from_string")]
            pub nowplaying: Option<bool>,
        }
    }
}

fn bool_from_string<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(match s.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    })
}
