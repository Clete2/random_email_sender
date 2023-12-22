use random_email_sender::{
    configuration::{Configuration, Imap, ReadAction},
    error::Error,
};

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let configuration = Configuration::load().expect("Could not load configuration.");

    if let Some(read_configuration) = &configuration.read {
        return consume_messages(read_configuration).await;
    } else {
        return Err(Error {
            message: "Asked to consume messages, but no configuration was found!".to_string(),
        });
    }
}

pub async fn consume_messages(configuration: &Imap) -> Result<(), Error> {
    let tls = native_tls::TlsConnector::builder().build().unwrap();

    let client = imap::connect(
        (configuration.server.as_str(), 993),
        &configuration.server,
        &tls,
    )?;

    let mut session = client
        .login(&configuration.username, &configuration.password)
        .map_err(|e| e.0)?; // Error is a tuple (Error, UnauthenticatedClient); map to just Error

    session.select("INBOX")?;

    let uids = session.uid_search("1:* NOT SEEN")?;

    let mut failures = vec![];

    for uid in &uids {
        let uid = uid.to_string();
        let result: Result<(), Error> = match &configuration.action {
            ReadAction::Read => match session.uid_store(uid, "+FLAGS (\\Seen)") {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            },
            ReadAction::Delete => match session.uid_expunge(uid) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            },
        };
        if let Err(error) = result {
            failures.push(error.to_string())
        }
    }

    if !failures.is_empty() {
        Err(Error {
            message: format!(
                "{} of {} emails failed to {:?}! Errors follow: {:#?}",
                failures.len(),
                &uids.len(),
                &configuration.action,
                failures
            ),
        })
    } else {
        Ok(())
    }
}
