use std::sync::LazyLock;

use demand::Input;
use regex::Regex;
use reqwest::{Client as ReqwestClient, StatusCode};
use revolt_models::{paths::auth::session::login, schemas};

use crate::Command;

const SUCCESSFUL_LOGIN_RESPONSE_TEMPLATE: fn(&str) -> String = |session_token| {
    format!(
        r#"
✓ Session token successfully acquired!

To complete setup:
1. Open your configuration file
2. Locate the `revolt: session_token:` line
3. Replace it with:
   session_token: "{session_token}"

If you're connecting to a custom Revolt instance (using `api-url` flag), 
also update the following in your config:
revolt: api_url: <instance-url>

⚠️ SECURITY WARNING
Your session token grants complete access to your account.
Never share it with anyone and store it securely, by using
`_file` or `_FILE` suffix in your configuration file and
environment variables respectively.
"#
    )
};

static REVOLT_SESSION_FRIENDLY_NAME: LazyLock<String> = LazyLock::new(|| {
    format!(
        "lure on {os}{repo}",
        os = std::env::consts::OS,
        repo = env!("CARGO_PKG_REPOSITORY")
            .strip_prefix("https://")
            .map(|repo| format!(" ({repo})"))
            .unwrap_or_default()
    )
});

static EMAIL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

static REVOLT_RECOVERY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^([a-z0-9]{5}-[a-z0-9]{5})$").unwrap());

static REVOLT_TOTP_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[0-9]{6}$").unwrap());

#[derive(Debug, clap::Subcommand)]
pub enum Subcommands {
    /// Generate an example lure configuration file and print it.
    Generate,
    /// Revolt commands for obtaining some configuration options.
    #[command(subcommand)]
    Revolt(RevoltSubcommands),
}

#[derive(Debug, clap::Subcommand)]
pub enum RevoltSubcommands {
    /// Login to Revolt to obtain a new session token.
    GetSessionToken {
        #[arg(long, default_value = "https://api.revolt.chat")]
        api_url: String,
    },
}

impl Command for Subcommands {
    type Error = SubcommandError;

    async fn run(&self) -> Result<(), Self::Error> {
        match self {
            Self::Generate => {
                Self::generate();
                Ok(())
            }
            Self::Revolt(subcommand) => match subcommand.run().await {
                Ok(result) => Ok(result),
                Err(error) => Err(error.into()),
            },
        }
    }
}

impl Subcommands {
    pub fn generate() {
        print!("{}", lure_resources::CONFIG_SAMPLE_FILE);
    }
}

impl RevoltSubcommands {
    async fn run(&self) -> Result<(), RevoltSubcommandsError> {
        match self {
            Self::GetSessionToken { api_url } => Self::get_session_token(api_url).await,
        }
    }

    pub async fn get_session_token(revolt_api_url: &str) -> Result<(), RevoltSubcommandsError> {
        let client = ReqwestClient::new();

        let email = match Input::new("E-mail:")
            .placeholder("i@love.cat")
            .validation(|value| {
                if EMAIL_REGEX.is_match(value) {
                    return Ok(());
                }

                Err("Invalid e-mail address.")
            })
            .run()
        {
            Ok(email) => email,
            Err(error) => {
                if error.kind() == std::io::ErrorKind::Interrupted {
                    return Ok(());
                }

                return Err(error.into());
            }
        };

        let password = match Input::new("Password:")
            .description(
                "Password won't be stored anywhere and is only used to obtain a session token.",
            )
            .password(true)
            .placeholder("JaneDoe0")
            .validation(|value| {
                if value.is_empty() {
                    return Err("Password cannot be empty.");
                }

                Ok(())
            })
            .run()
        {
            Ok(password) => password,
            Err(error) => {
                if error.kind() == std::io::ErrorKind::Interrupted {
                    return Ok(());
                }

                return Err(error.into());
            }
        };

        match client
            .post(format!("{revolt_api_url}/auth/session/login"))
            .json(&login::RequestBody::Login {
                email,
                password,
                friendly_name: Some(REVOLT_SESSION_FRIENDLY_NAME.to_string()),
            })
            .send()
            .await?
            .handle_user_friendly_error()
            .await?
            .json::<login::ResponseBody>()
            .await?
        {
            login::ResponseBody::Success { token, name: _ } => Self::on_login_success(&token),
            login::ResponseBody::MFA {
                ticket,
                allowed_methods,
            } => Self::on_login_mfa(ticket, &allowed_methods, &client, revolt_api_url).await?,
            login::ResponseBody::Disabled => Self::on_login_disabled(),
        }

        Ok(())
    }

    fn on_login_success(token: &str) {
        println!("{}", SUCCESSFUL_LOGIN_RESPONSE_TEMPLATE(token));
    }

    async fn on_login_mfa(
        ticket: String,
        allowed_methods: &[schemas::mfa::Method],
        client: &ReqwestClient,
        revolt_api_url: &str,
    ) -> Result<(), RevoltSubcommandsError> {
        let mfa_prompt = if allowed_methods.len() > 1 {
            Input::new("Enter 2FA authentication or recovery code:").validation(|value| {
                if REVOLT_RECOVERY_REGEX.is_match(value) || REVOLT_TOTP_REGEX.is_match(value) {
                    return Ok(());
                }

                Err("Invalid recovery or TOTP code.")
            })
        } else {
            Input::new("Enter 2FA authentication code:").validation(|value| {
                if REVOLT_TOTP_REGEX.is_match(value) {
                    return Ok(());
                }

                Err("Invalid TOTP code.")
            })
        };

        let mfa_data = match mfa_prompt.run() {
            Ok(mfa_code) => {
                if REVOLT_TOTP_REGEX.is_match(&mfa_code) {
                    schemas::mfa::Response::TotpCode {
                        totp_code: mfa_code,
                    }
                } else {
                    schemas::mfa::Response::RecoveryCode {
                        recovery_code: mfa_code,
                    }
                }
            }
            Err(error) => {
                if error.kind() == std::io::ErrorKind::Interrupted {
                    return Ok(());
                }

                return Err(error.into());
            }
        };

        match client
            .post(format!("{revolt_api_url}/auth/session/login"))
            .json(&login::RequestBody::MFA {
                mfa_ticket: ticket,
                mfa_response: Some(mfa_data),
                friendly_name: Some(REVOLT_SESSION_FRIENDLY_NAME.to_string()),
            })
            .send()
            .await?
            .handle_user_friendly_error()
            .await?
            .json::<login::ResponseBody>()
            .await?
        {
            login::ResponseBody::Success { token, name: _ } => Self::on_login_success(&token),
            _ => unreachable!(),
        }

        Ok(())
    }

    fn on_login_disabled() {
        println!("Login is disabled on this account.");
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SubcommandError {
    #[error(transparent)]
    RevoltSubcommands(#[from] RevoltSubcommandsError),
}

#[derive(Debug, thiserror::Error)]
pub enum RevoltSubcommandsError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    ExpectedAuthError(#[from] ExpectedAuthError),
    #[error("Unexpected status code received: {0}")]
    UnexpectedAuthStatusCode(StatusCode),
}

#[derive(Debug, thiserror::Error)]
pub enum ExpectedAuthError {
    #[error("The account you are trying to log in to is unverified.")]
    UnverifiedAccount,
    #[error("Incorrect 2FA code provided.")]
    InvalidToken,
    #[error("Invalid login credentials provided.")]
    InvalidCredentials,
    #[error(
        "The entered password is compromised. Please ensure you have entered the correct password."
    )]
    CompromisedPassword,
    #[error(
        "The entered password is too short. Please ensure you have entered the correct password."
    )]
    ShortPassword,
    #[error("The entered email is blacklisted. Please ensure you have entered the correct email.")]
    Blacklisted,
    #[error("This account is locked out. Please try again some time later.")]
    LockedOut,
}

impl From<schemas::AuthifierError> for ExpectedAuthError {
    fn from(value: schemas::AuthifierError) -> Self {
        match value {
            schemas::AuthifierError::UnverifiedAccount => Self::UnverifiedAccount,
            schemas::AuthifierError::InvalidToken => Self::InvalidToken,
            schemas::AuthifierError::InvalidCredentials => Self::InvalidCredentials,
            schemas::AuthifierError::CompromisedPassword => Self::CompromisedPassword,
            schemas::AuthifierError::ShortPassword => Self::ShortPassword,
            schemas::AuthifierError::Blacklisted => Self::Blacklisted,
            schemas::AuthifierError::LockedOut => Self::LockedOut,
        }
    }
}

trait ReqwestResponseExt: Sized {
    async fn handle_user_friendly_error(self) -> Result<Self, RevoltSubcommandsError>;
}

impl ReqwestResponseExt for reqwest::Response {
    async fn handle_user_friendly_error(self) -> Result<Self, RevoltSubcommandsError> {
        match self.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => Ok(self),
            StatusCode::BAD_REQUEST | StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                let error = self.json::<schemas::AuthifierError>().await?;
                Err(ExpectedAuthError::from(error).into())
            }
            error => Err(RevoltSubcommandsError::UnexpectedAuthStatusCode(error)),
        }
    }
}
