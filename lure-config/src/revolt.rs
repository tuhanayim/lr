#[derive(Debug, serde::Deserialize)]
pub struct Options {
    #[serde(default = "default_revolt_status")]
    pub status: StatusOptions,
    #[serde(default = "default_revolt_api_url")]
    pub api_url: String,
    pub session_token: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct StatusOptions {
    #[serde(default = "default_revolt_status_template")]
    pub template: String,
    #[serde(default)]
    pub idle: Option<String>,
}

fn default_revolt_status() -> StatusOptions {
    StatusOptions {
        template: default_revolt_status_template(),
        idle: None,
    }
}

fn default_revolt_status_template() -> String {
    String::from("🎵 Listening to %NAME% by %ARTIST%")
}

fn default_revolt_api_url() -> String {
    String::from("https://api.revolt.chat")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_minimal() {
        let yaml = r#"
            session_token: meow
        "#;

        let options: Options = serde_yml::from_str(&yaml).unwrap();

        assert_eq!(options.api_url, "https://api.revolt.chat");
        assert_eq!(options.session_token, "meow");
        assert_eq!(
            options.status.template,
            "🎵 Listening to %NAME% by %ARTIST%"
        );
        assert_eq!(options.status.idle, None);
    }

    #[test]
    fn test_options_full() {
        let yaml = r#"
            api_url: https://api.kittenvolt.cat
            session_token: mrrp
            status:
                template: "%NAME% by %ARTIST%"
                idle: Not listening to anything!
        "#;

        let options: Options = serde_yml::from_str(&yaml).unwrap();

        assert_eq!(options.api_url, "https://api.kittenvolt.cat");
        assert_eq!(options.session_token, "mrrp");
        assert_eq!(options.status.template, "%NAME% by %ARTIST%");
        assert_eq!(
            options.status.idle,
            Some("Not listening to anything!".to_string())
        );
    }

    #[test]
    #[should_panic(expected = "missing field `session_token`")]
    fn test_missing_session_token() {
        let _: Options = serde_yml::from_str("").unwrap();
    }
}
