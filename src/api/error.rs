use std::fmt;

#[derive(Debug)]
pub enum ApiError {
    Network(reqwest::Error),
    Serialization(serde_json::Error),
    Unauthorized,
    NotFound,
    Server(String),
    Other(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(e) => write!(f, "Network error: {e}"),
            Self::Serialization(e) => write!(f, "JSON error: {e}"),
            Self::Unauthorized => write!(f, "Unauthorized — check your access token"),
            Self::NotFound => write!(f, "Not found"),
            Self::Server(msg) => write!(f, "Server error: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        Self::Network(e)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e)
    }
}
