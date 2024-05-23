use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Invalid Language: {0}")]
    InvalidLanguage(String),

    #[error("Unknown Error: {0}")]
    Unknown(String),
}
