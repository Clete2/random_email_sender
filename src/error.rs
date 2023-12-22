use std::fmt::Display;

use imap::Error as ImapError;
use serde_json::error::Error as SerdeError;

impl From<SerdeError> for Error {
    fn from(e: SerdeError) -> Self {
        Self {
            message: format!("Could not deserialize config file. Error: {}", e),
        }
    }
}

impl From<ImapError> for Error {
    fn from(e: ImapError) -> Self {
        Self {
            message: format!("Could not read emails from IMAP server. Error: {}", e),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error: {}", &self.message))
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct Error {
    pub message: String,
}
