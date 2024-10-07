use std::{array, error, fmt, io, num, time};

#[derive(Debug)]
pub enum PusherError {
    Axum(axum::Error),
    FromSlice(array::TryFromSliceError),
    Header(reqwest::header::InvalidHeaderValue),
    Io(io::Error),
    Migrate(sqlx::migrate::MigrateError),
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
    Sqlx(sqlx::Error),
    SysTime(time::SystemTimeError),
    OpenSSL(openssl::error::ErrorStack),
    Other(String),
    ParseIntError(num::ParseIntError),
    UrlParseError(url::ParseError),
}

pub type Res<T> = Result<T, PusherError>;

impl fmt::Display for PusherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PusherError::Axum(e) => write!(f, "{}", e),
            PusherError::FromSlice(e) => write!(f, "{}", e),
            PusherError::Header(e) => write!(f, "{}", e),
            PusherError::Io(e) => write!(f, "{}", e),
            PusherError::Migrate(e) => write!(f, "{}", e),
            PusherError::Reqwest(e) => write!(f, "{}", e),
            PusherError::SerdeJson(e) => write!(f, "{}", e),
            PusherError::Sqlx(e) => write!(f, "{}", e),
            PusherError::SysTime(e) => write!(f, "{}", e),
            PusherError::OpenSSL(e) => write!(f, "{}", e),
            PusherError::Other(e) => write!(f, "{}", e),
            PusherError::ParseIntError(e) => write!(f, "{}", e),
            PusherError::UrlParseError(e) => write!(f, "{}", e),
        }
    }
}

impl error::Error for PusherError {}

impl From<axum::Error> for PusherError {
    fn from(value: axum::Error) -> Self {
        Self::Axum(value)
    }
}

impl From<array::TryFromSliceError> for PusherError {
    fn from(value: array::TryFromSliceError) -> Self {
        Self::FromSlice(value)
    }
}

impl From<io::Error> for PusherError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<reqwest::Error> for PusherError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<reqwest::header::InvalidHeaderValue> for PusherError {
    fn from(value: reqwest::header::InvalidHeaderValue) -> Self {
        Self::Header(value)
    }
}

impl From<sqlx::migrate::MigrateError> for PusherError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        Self::Migrate(value)
    }
}

impl From<serde_json::Error> for PusherError {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
    }
}

impl From<openssl::error::ErrorStack> for PusherError {
    fn from(value: openssl::error::ErrorStack) -> Self {
        Self::OpenSSL(value)
    }
}

impl From<sqlx::Error> for PusherError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(value)
    }
}

impl From<time::SystemTimeError> for PusherError {
    fn from(value: time::SystemTimeError) -> Self {
        Self::SysTime(value)
    }
}

impl From<String> for PusherError {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

impl From<&str> for PusherError {
    fn from(value: &str) -> Self {
        Self::Other(value.to_owned())
    }
}

impl From<num::ParseIntError> for PusherError {
    fn from(value: num::ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

impl From<url::ParseError> for PusherError {
    fn from(value: url::ParseError) -> Self {
        Self::UrlParseError(value)
    }
}

impl axum::response::IntoResponse for PusherError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{}", self);
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Something went wrong".to_string(),
        )
            .into_response()
    }
}
