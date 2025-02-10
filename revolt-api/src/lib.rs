use core::{future::Future, str::FromStr as _};

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    StatusCode,
};
use revolt_models::{
    schemas::user::{DataEditUser, FieldsUser, User, UserStatus},
    Authentication,
};

pub struct Client {
    http_client: reqwest::Client,
    base_url: String,
}

impl Client {
    pub fn try_new(api_url: String, authentication: &Authentication) -> Result<Self, Error> {
        let headers = HeaderMap::from_iter([(
            HeaderName::from_str(authentication.header())?,
            HeaderValue::from_str(&authentication.value())?,
        )]);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            http_client: client,
            base_url: api_url,
        })
    }

    pub async fn get_status_text(&self) -> Result<Option<String>, Error> {
        let response: User = self
            .http_client
            .get(format!("{}/users/@me", self.base_url))
            .send()
            .await?
            .handle_return_error()
            .await?
            .json()
            .await?;

        let status = response.status.and_then(|status| status.text);

        Ok(status)
    }

    pub async fn set_status_text(&self, status_text: Option<String>) -> Result<(), Error> {
        let data = status_text.map_or_else(
            || DataEditUser {
                remove: Some(vec![FieldsUser::StatusText]),
                ..Default::default()
            },
            |status_text| DataEditUser {
                status: Some(UserStatus {
                    text: Some(status_text),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );

        self.http_client
            .patch(format!("{}/users/@me", self.base_url))
            .json(&data)
            .send()
            .await?
            .handle_return_error()
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ApiError(#[from] APIError),
    #[error(transparent)]
    HeaderName(#[from] reqwest::header::InvalidHeaderName),
    #[error(transparent)]
    HeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum APIError {
    #[error("Revolt API authentication failed. Please check your credentials.")]
    AuthenticationFailed,
    #[error("Revolt API rate limit exceeded.")]
    RateLimitExceeded(u64),
    #[error("Revolt API returned an unexpected error: {0}")]
    Unknown(String),
}

pub trait HandleAPIError: Sized {
    type Error: core::error::Error;

    fn handle_return_error(self) -> impl Future<Output = Result<Self, Self::Error>>;
}

impl HandleAPIError for reqwest::Response {
    type Error = APIError;

    async fn handle_return_error(self) -> Result<Self, Self::Error> {
        match self.status() {
            StatusCode::OK => Ok(self),
            StatusCode::UNAUTHORIZED => Err(APIError::AuthenticationFailed),
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = self
                    .headers()
                    .get("X-Ratelimit-Reset-After")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.parse::<u64>().ok())
                    .unwrap_or(0);

                Err(APIError::RateLimitExceeded(retry_after))
            }
            status => Err(APIError::Unknown(format!(
                "Unexpected HTTP status: {status}"
            ))),
        }
    }
}
