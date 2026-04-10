use thiserror::Error;

#[derive(Error, Debug)]
pub enum AzrsError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Authentication error with a re-login suggestion (matches az Python behavior).
    /// `message` is the AAD error description; `suggestion` is the `azrs logout\nazrs login ...` command.
    #[error("{message}")]
    AuthWithSuggestion {
        message: String,
        suggestion: String,
    },

    #[error("Token expired and no refresh token available")]
    #[allow(dead_code)]
    TokenExpired,

    #[error("Profile error: {0}")]
    #[allow(dead_code)]
    Profile(String),

    #[error("No active subscription found. Run 'azrs login' first.")]
    NoActiveSubscription,

    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(String),

    #[error("{0}")]
    General(String),
}

impl AzrsError {
    /// Get the re-login suggestion if this error carries one.
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            AzrsError::AuthWithSuggestion { suggestion, .. } => Some(suggestion),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, AzrsError>;
