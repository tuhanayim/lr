pub mod paths {
    pub mod auth {
        pub mod session {
            pub mod login {
                use crate::schemas::mfa;

                #[derive(Debug, serde::Deserialize)]
                #[serde(tag = "result")]
                pub enum ResponseBody {
                    Success {
                        token: String,
                        name: String,
                    },
                    MFA {
                        ticket: String,
                        allowed_methods: Vec<mfa::Method>,
                    },
                    Disabled,
                }

                #[derive(Debug, serde::Serialize)]
                #[serde(untagged)]
                pub enum RequestBody {
                    Login {
                        email: String,
                        password: String,
                        #[serde(skip_serializing_if = "Option::is_none")]
                        friendly_name: Option<String>,
                    },
                    MFA {
                        mfa_ticket: String,
                        #[serde(skip_serializing_if = "Option::is_none")]
                        mfa_response: Option<mfa::Response>,
                        #[serde(skip_serializing_if = "Option::is_none")]
                        friendly_name: Option<String>,
                    },
                }
            }
        }
    }
}

pub mod schemas {
    pub mod mfa {
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        #[serde(untagged)]
        pub enum Response {
            Password { password: String },
            RecoveryCode { recovery_code: String },
            TotpCode { totp_code: String },
        }

        #[derive(Debug, serde::Deserialize)]
        pub enum Method {
            Password,
            Recovery,
            Totp,
        }
    }

    pub mod user {
        #[derive(Debug, Default, serde::Serialize)]
        pub struct DataEditUser {
            #[serde(skip_serializing_if = "Option::is_none")]
            pub status: Option<UserStatus>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub remove: Option<Vec<FieldsUser>>,
        }

        #[derive(Debug, serde::Deserialize)]
        pub struct User {
            pub status: Option<UserStatus>,
        }

        #[derive(Debug, serde::Serialize)]
        pub enum FieldsUser {
            StatusText,
        }

        #[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
        pub struct UserStatus {
            #[serde(skip_serializing_if = "Option::is_none")]
            pub text: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub presence: Option<Presence>,
        }

        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub enum Presence {
            Online,
            Idle,
            Focus,
            Busy,
            Invisible,
        }
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(tag = "type")]
    pub enum AuthifierError {
        UnverifiedAccount,
        InvalidToken,
        InvalidCredentials,
        CompromisedPassword,
        ShortPassword,
        Blacklisted,
        LockedOut,
    }
}

#[derive(Debug)]
pub enum Authentication {
    SessionToken(String),
}

impl Authentication {
    #[must_use]
    pub const fn header(&self) -> &str {
        match self {
            Self::SessionToken(_) => "X-Session-Token",
        }
    }

    #[must_use]
    pub fn value(&self) -> String {
        match self {
            Self::SessionToken(token) => token,
        }
        .to_owned()
    }
}
