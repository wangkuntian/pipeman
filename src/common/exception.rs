use ini::Error as IniError;
use reqwest::Error as RequestError;
use serde_json::Error as SerdeError;
use ssh2::Error as SSHError;
use std::io::Error as IOError;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Exception {
    #[error("{0}")]
    RequestException(#[from] RequestError),

    #[error("{0}")]
    SerdeException(#[from] SerdeError),

    #[error("authentication error")]
    AuthenticationException,

    #[error("{0}")]
    SSHException(#[from] SSHError),

    #[error("{0}")]
    IOException(#[from] IOError),

    #[error("{0}")]
    FromUtf8Exception(#[from] FromUtf8Error),

    #[error("{0}")]
    IniException(#[from] IniError),

    #[error("{0}")]
    Error(String),
}

pub type Result<T> = std::result::Result<T, Exception>;
