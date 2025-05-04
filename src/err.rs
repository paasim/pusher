use std::{array, error, fmt, io, num, time};

#[derive(Debug)]
pub enum PusherError {
    Axum(axum::Error),
    DeadpoolCreate(deadpool_sqlite::CreatePoolError),
    DeadpoolInteract(deadpool_sqlite::InteractError),
    DeadpoolPool(deadpool_sqlite::PoolError),
    FromSlice(array::TryFromSliceError),
    Header(reqwest::header::InvalidHeaderValue),
    Io(io::Error),
    Reqwest(reqwest::Error),
    Rusqlite(deadpool_sqlite::rusqlite::Error),
    SerdeJson(serde_json::Error),
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
            PusherError::DeadpoolCreate(e) => write!(f, "{}", e),
            PusherError::DeadpoolInteract(e) => write!(f, "{}", e),
            PusherError::DeadpoolPool(e) => write!(f, "{}", e),
            PusherError::FromSlice(e) => write!(f, "{}", e),
            PusherError::Header(e) => write!(f, "{}", e),
            PusherError::Io(e) => write!(f, "{}", e),
            PusherError::Reqwest(e) => write!(f, "{}", e),
            PusherError::Rusqlite(e) => write!(f, "{}", e),
            PusherError::SerdeJson(e) => write!(f, "{}", e),
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

impl From<deadpool_sqlite::CreatePoolError> for PusherError {
    fn from(value: deadpool_sqlite::CreatePoolError) -> Self {
        Self::DeadpoolCreate(value)
    }
}

impl From<deadpool_sqlite::InteractError> for PusherError {
    fn from(value: deadpool_sqlite::InteractError) -> Self {
        Self::DeadpoolInteract(value)
    }
}

impl From<deadpool_sqlite::PoolError> for PusherError {
    fn from(value: deadpool_sqlite::PoolError) -> Self {
        Self::DeadpoolPool(value)
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

impl From<deadpool_sqlite::rusqlite::Error> for PusherError {
    fn from(value: deadpool_sqlite::rusqlite::Error) -> Self {
        Self::Rusqlite(value)
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
