use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AppError {
    MissingHomeDirectory,
    MissingLocalAppData,
    MissingChromeCacheDirectory(String),
    Io(std::io::Error),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::MissingHomeDirectory => write!(f, "Unable to determine the user home directory."),
            AppError::MissingLocalAppData => write!(f, "Unable to determine LOCALAPPDATA path."),
            AppError::MissingChromeCacheDirectory(path) => {
                write!(f, "Chrome cache directory was not found: {path}")
            }
            AppError::Io(err) => write!(f, "I/O error: {err}"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::Io(value)
    }
}
