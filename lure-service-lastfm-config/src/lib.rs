use secrecy::SecretString;

#[derive(Debug, serde::Deserialize)]
pub struct Options {
    /// Enable the service.
    #[serde(default = "default_enable")]
    pub enable: bool,
    /// `Last.fm` username to check for listening activity.
    pub username: String,
    /// `Last.fm` API key to use for checking listening activity.
    pub api_key: SecretString,
    /// Interval in seconds to check for listening activity.
    #[serde(default = "default_check_interval")]
    pub check_interval: u64,
}

const fn default_enable() -> bool {
    false
}

const fn default_check_interval() -> u64 {
    16
}

#[cfg(test)]
mod tests {
    use secrecy::ExposeSecret;

    use super::*;

    #[test]
    fn test_options_minimal() {
        let yaml = r#"
            enable: true
            username: kitty
            api_key: hellokitty
        "#;

        let options: Options = serde_yml::from_str(&yaml).unwrap();

        assert!(options.enable);

        assert_eq!(options.username, "kitty");
        assert_eq!(options.api_key.expose_secret(), "hellokitty");
        assert_eq!(options.check_interval, 16);
    }

    #[test]
    fn test_options_full() {
        let yaml = r#"
            username: kitten
            api_key: hellokitten
            check_interval: 24
        "#;

        let options: Options = serde_yml::from_str(&yaml).unwrap();

        assert!(!options.enable);

        assert_eq!(options.username, "kitten");
        assert_eq!(options.api_key.expose_secret(), "hellokitten");
        assert_eq!(options.check_interval, 24);
    }

    #[test]
    #[should_panic(expected = "missing field `username`")]
    fn test_missing_username() {
        let _: Options = serde_yml::from_str("").unwrap();
    }

    #[test]
    #[should_panic(expected = "missing field `api_key`")]
    fn test_missing_api_key() {
        let _: Options = serde_yml::from_str("username: cat").unwrap();
    }
}
