use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Http(reqwest::Error),
    Spider(Box<dyn std::error::Error + Send + Sync>),
    Parse(url::ParseError),
    ParseInt(std::num::ParseIntError),
    Custom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Http(e) => write!(f, "HTTP error: {}", e),
            Error::Spider(e) => write!(f, "Spider error: {}", e),
            Error::Parse(e) => write!(f, "URL parse error: {}", e),
            Error::ParseInt(e) => write!(f, "Parse int error: {}", e),
            Error::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Http(error)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Error::Spider(error)
    }
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Self {
        Error::Parse(error)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Error::ParseInt(error)
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::Custom(error)
    }
}

impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Error::Custom(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;