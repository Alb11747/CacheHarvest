use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AppError {
    MissingHomeDirectory,
    MissingLocalAppData,
    MissingChromeCacheDirectories(Vec<String>),
    OutputDirectoryCreate {
        path: String,
        source: std::io::Error,
    },
    Io(std::io::Error),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::MissingHomeDirectory => write!(f, "Unable to determine the user home directory."),
            AppError::MissingLocalAppData => write!(f, "Unable to determine LOCALAPPDATA path."),
            AppError::MissingChromeCacheDirectories(paths) => {
                write!(
                    f,
                    "Chrome cache directories were not found. Checked: {}",
                    paths.join(", ")
                )
            }
            AppError::OutputDirectoryCreate { path, source } => {
                write!(f, "Unable to create export directory '{path}': {source}")
            }
            AppError::Io(err) => write!(f, "I/O error: {err}"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::OutputDirectoryCreate { source, .. } => Some(source),
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
