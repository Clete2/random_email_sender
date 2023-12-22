use crate::error::Error;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::{fs::File, str::FromStr, sync::Mutex};

#[derive(Deserialize, PartialEq, Debug, Clone)]
// https://serde.rs/variant-attrs.html
#[serde(rename_all = "camelCase")]
// Configuration for email sending
// smtp_password defaults to the smtpPassword env var if not populated
pub struct Configuration {
    pub email_path: String,
    pub file_type: EmailFileType,
    pub smtp: Smtp,
    // Who to send errors to; best to configure a totally separate SMTP connection
    pub error_smtp: Option<Smtp>,
    pub read: Option<Imap>,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum EmailFileType {
    Lzma,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
// https://serde.rs/variant-attrs.html
#[serde(rename_all = "camelCase")]
pub struct Imap {
    pub server: String,
    pub username: String,
    #[serde(default = "default_imap_password")]
    pub password: String,
    pub action: ReadAction,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ReadAction {
    Read,
    Delete,
}

impl FromStr for ReadAction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        if s == "read" {
            Ok(ReadAction::Read)
        } else if s == "delete" {
            Ok(ReadAction::Delete)
        } else {
            Err(Error {
                message: "Could not parse".to_string(),
            })
        }
    }
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
// https://serde.rs/variant-attrs.html
#[serde(rename_all = "camelCase")]
pub struct Smtp {
    pub server: String,
    pub username: String,
    #[serde(default = "default_smtp_password")] // Prevent errors
    pub password: String,
    pub from: String,
    pub reply_to: String,
    pub to: String,
}

static DEFAULT_PASSWORD: &str = "DEFAULT_PASSWORD23536735467452341234"; // TODO: Bad design choice

fn default_smtp_password() -> String {
    DEFAULT_PASSWORD.to_string()
}

fn default_imap_password() -> String {
    DEFAULT_PASSWORD.to_string()
}

lazy_static! {
    static ref CONFIGURATION: Mutex<Option<Configuration>> = Mutex::new(None);
}

impl Configuration {
    // Retrieves a copy of the configuration singleton
    // or loads from one of the default location
    pub fn load() -> Result<Configuration, Error> {
        if let Ok(config_var) = std::env::var("randomEmailSenderConfigLocation") {
            if let Ok(configuration) = Self::load_from_location(config_var.as_str()) {
                return Ok(configuration);
            }
        }

        if let Ok(configuration) = Self::load_from_location("./config.json") {
            return Ok(configuration);
        }

        let mut config_location =
            std::env::current_exe().expect("Could not determine current directory.");
        config_location.pop(); // Remove filename
        config_location.push("config.json");
        let config_location = config_location
            .to_str()
            .expect("Could not convert directory to string.");
        if let Ok(configuration) = Self::load_from_location(config_location) {
            return Ok(configuration);
        }

        Err(Error {
            message:
                "Could not load from ./config.json or from env var randomEmailSenderConfigLocation!"
                    .to_string(),
        })
    }

    // Retrieves a copy of the configuration singleton
    // From a specified location
    pub fn load_from_location(location: &str) -> Result<Configuration, Error> {
        if let Some(configuration) = CONFIGURATION
            .lock()
            .expect("Could not lock configuration mutex")
            .to_owned()
        {
            return Ok(configuration);
        }

        if let Ok(file) = File::open(location) {
            let mut configuration: Configuration = serde_json::from_reader(file)?;
            if configuration.smtp.password == *DEFAULT_PASSWORD {
                configuration.smtp.password = std::env::var("smtpPassword").expect(
                    "SMTP password must be set, or smtpPassword environment variable must be set.",
                );
            }
            if let Some(read) = &mut configuration.read {
                if read.password == *DEFAULT_PASSWORD {
                    read.password = std::env::var("readPassword").expect("Reading password must be set, or readPassword environment variable must be set.");
                }
            }

            if let Some(error_configuration) = &mut configuration.error_smtp {
                if error_configuration.password == *DEFAULT_PASSWORD {
                    error_configuration.password = std::env::var("smtpErrorPassword").expect("SMTP Error password must be set when SMTP error functionality is enabled. Either set it in the configuration or set the smtpErrorPassword environment variable.");
                }
            }

            *CONFIGURATION.lock().unwrap() = Some(configuration.clone());
            Ok(configuration)
        } else {
            Err(Error {
                message: String::from("Could not find configuration file."),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_local_config() {
        std::env::set_var("smtpPassword", "MyPassword");
        std::env::set_var("smtpErrorPassword", "SecondPassword");
        std::env::set_var("readPassword", "ThirdPassword");
        insta::assert_debug_snapshot!(Configuration::load_from_location("config.json").unwrap());
    }

    #[test]
    fn can_get_twice() {
        assert_eq!(
            Configuration::load().unwrap(),
            Configuration::load().unwrap()
        )
    }

    #[test]
    fn succeeds_on_second_call_even_with_invalid_path() {
        // TODO: Poor design choice; should refactor
        assert_eq!(
            Configuration::load().unwrap(),
            Configuration::load_from_location("invalid.json").unwrap()
        )
    }

    #[test]
    fn load_from_specific_location() {
        std::env::set_var("smtpPassword", "MyPassword");
        std::env::set_var("smtpErrorPassword", "SecondPassword");
        std::env::set_var("readPassword", "ThirdPassword");
        insta::assert_debug_snapshot!(Configuration::load_from_location("config.json").unwrap());
    }

    #[test]
    fn fails_to_load_from_specific_location() {
        Configuration::load_from_location("invalid.json").expect_err("Did not fail to load!");
    }
}
