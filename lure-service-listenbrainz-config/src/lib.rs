#[derive(Debug, serde::Deserialize)]
pub struct Options {
    /// Enable the service.
    #[serde(default = "default_enable")]
    pub enable: bool,
    /// `ListenBrainz` username to check for listening activity.
    pub username: String,
    /// `ListenBrainz` API URL to use for checking listening activity.
    #[serde(default = "default_listenbrainz_api_url")]
    pub api_url: String,
    /// Interval in seconds to check for listening activity.
    #[serde(default = "default_check_interval")]
    pub check_interval: u64,
}

const fn default_enable() -> bool {
    false
}

fn default_listenbrainz_api_url() -> String {
    String::from("https://api.listenbrainz.org")
}

const fn default_check_interval() -> u64 {
    16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_minimal() {
        let yaml = r#"
            enable: true
            username: kitty
        "#;

        let options: Options = serde_yml::from_str(&yaml).unwrap();

        assert!(options.enable);

        assert_eq!(options.username, "kitty");
        assert_eq!(options.api_url, "https://api.listenbrainz.org");
        assert_eq!(options.check_interval, 16);
    }

    #[test]
    fn test_options_full() {
        let yaml = r#"
            username: kitten
            api_url: https://api.kittenbrainz.cat
            check_interval: 24
        "#;

        let options: Options = serde_yml::from_str(&yaml).unwrap();

        assert!(!options.enable);

        assert_eq!(options.username, "kitten");
        assert_eq!(options.api_url, "https://api.kittenbrainz.cat");
        assert_eq!(options.check_interval, 24);
    }

    #[test]
    #[should_panic(expected = "missing field `username`")]
    fn test_missing_required_fields() {
        let _: Options = serde_yml::from_str("").unwrap();
    }
}
