#![cfg(any(feature = "service-lastfm", feature = "service-listenbrainz"))]

pub mod revolt;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub service: ServiceOptions,
    pub revolt: revolt::Options,
}

#[derive(Debug, serde::Deserialize)]
pub struct ServiceOptions {
    #[cfg(feature = "service-lastfm")]
    pub lastfm: Option<lure_service_lastfm_config::Options>,
    #[cfg(feature = "service-listenbrainz")]
    pub listenbrainz: Option<lure_service_listenbrainz_config::Options>,
}

impl Config {
    #[must_use]
    pub fn enabled_services(&self) -> Vec<&str> {
        let mut services = Vec::new();

        #[cfg(feature = "service-lastfm")]
        if let Some(lastfm) = &self.service.lastfm {
            if lastfm.enable {
                services.push("LastFM");
            }
        }

        #[cfg(feature = "service-listenbrainz")]
        if let Some(listenbrainz) = &self.service.listenbrainz {
            if listenbrainz.enable {
                services.push("ListenBrainz");
            }
        }

        services
    }
}
