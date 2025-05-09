use std::{error, fmt, io};

#[derive(Debug)]
pub enum Error {
    Axum(axum::Error),
    DeadpoolCreate(deadpool_sqlite::CreatePoolError),
    DeadpoolInteract(deadpool_sqlite::InteractError),
    DeadpoolPool(deadpool_sqlite::PoolError),
    DeadpoolSqlite(deadpool_sqlite::rusqlite::Error),
    Header(reqwest::header::InvalidHeaderValue),
    Io(io::Error),
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
    OpenSSL(openssl::error::ErrorStack),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Axum(e) => write!(f, "{e}"),
            Error::DeadpoolCreate(e) => write!(f, "{e}"),
            Error::DeadpoolInteract(e) => write!(f, "{e}"),
            Error::DeadpoolPool(e) => write!(f, "{e}"),
            Error::DeadpoolSqlite(e) => write!(f, "{e}"),
            Error::Header(e) => write!(f, "{e}"),
            Error::Io(e) => write!(f, "{e}"),
            Error::Reqwest(e) => write!(f, "{e}"),
            Error::SerdeJson(e) => write!(f, "{e}"),
            Error::OpenSSL(e) => write!(f, "{e}"),
        }
    }
}

/// Matches on a Result, in case on Err, logs it, turns it a into response and returns it.
#[macro_export]
macro_rules! err_to_resp {
    ($e:expr) => {
        match $e {
            Ok(val) => val,
            Err(err) => {
                tracing::error!("{err}");
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_string(),
                )
                    .into_response();
            }
        }
    };
}

/// Wraps the error with [std::io::Error::other()], which can be turned into [Error] with ?.
#[macro_export]
macro_rules! err_other {
    ($e:expr) => {
        $e.map_err(|e| std::io::Error::other(e))
    };
    ($e:expr, $($arg:tt)*) => {
        $e.map_err(|_| std::io::Error::other(format!($($arg)*)))
    }
}

impl error::Error for Error {}

impl From<axum::Error> for Error {
    fn from(value: axum::Error) -> Self {
        Self::Axum(value)
    }
}

impl From<deadpool_sqlite::CreatePoolError> for Error {
    fn from(value: deadpool_sqlite::CreatePoolError) -> Self {
        Self::DeadpoolCreate(value)
    }
}

impl From<deadpool_sqlite::InteractError> for Error {
    fn from(value: deadpool_sqlite::InteractError) -> Self {
        Self::DeadpoolInteract(value)
    }
}

impl From<deadpool_sqlite::PoolError> for Error {
    fn from(value: deadpool_sqlite::PoolError) -> Self {
        Self::DeadpoolPool(value)
    }
}

impl From<deadpool_sqlite::rusqlite::Error> for Error {
    fn from(value: deadpool_sqlite::rusqlite::Error) -> Self {
        Self::DeadpoolSqlite(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<reqwest::header::InvalidHeaderValue> for Error {
    fn from(value: reqwest::header::InvalidHeaderValue) -> Self {
        Self::Header(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeJson(value)
    }
}

impl From<openssl::error::ErrorStack> for Error {
    fn from(value: openssl::error::ErrorStack) -> Self {
        Self::OpenSSL(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Io(io::Error::other(value))
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Io(io::Error::other(value))
    }
}
