use std::error::Error;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum CliError {
    NoMatchHandlerError,
    HandlerParamMissError,
    FileSysError(String),
    NetRequestError(String),
    UnknownError(String),
}

impl std::fmt::Display for CliError {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            CliError::NoMatchHandlerError => write!(f, "not handler matches the command!"),
            CliError::HandlerParamMissError => write!(f, "parser param miss error!"),
            CliError::FileSysError(err) => write!(f, "file sys error:{}", err),
            CliError::NetRequestError(err) => write!(f, "net request error:{}", err),
            CliError::UnknownError(err) => write!(f, "unknown error! {}", err),
        }
    }
}
impl Error for CliError {}
impl From<Box<dyn Error>> for CliError {
    fn from(e: Box<dyn Error>) -> Self {
        CliError::UnknownError(e.to_string())
    }
}
impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::FileSysError(e.to_string())
    }
}
impl From<reqwest::Error> for CliError {
    fn from(e: reqwest::Error) -> Self {
        CliError::NetRequestError(e.to_string())
    }
}
impl From<rodio::StreamError> for CliError {
    fn from(e: rodio::StreamError) -> Self {
        CliError::UnknownError(e.to_string())
    }
}
