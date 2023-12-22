use random_email_sender::{
    configuration::Configuration, email_format::Email, email_reader::get_random_email,
    email_sender::send_email, error::Error,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let configuration = Configuration::load().expect("Could not load configuration.");

    send_random_email(&configuration).await?;
    Ok(())
}

async fn send_random_email(configuration: &Configuration) -> Result<(), Error> {
    let email = get_random_email(&configuration.email_path);

    let email_result = send_email(&email, &configuration.smtp);

    if email_result.is_err() && configuration.error_smtp.is_some() {
        let mut headers = HashMap::new();
        headers.insert(
            "Subject".to_string(),
            "ERROR from Random Email Sender. Could not send email!".to_string(),
        );
        let email = Email {
            file: None,
            body: format!("Error: {:#?}", email_result.err().unwrap()),
            headers,
        };
        let error_result = send_email(&email, &configuration.error_smtp.clone().unwrap());

        let message = match error_result {
            Ok(_) => ", but successfully sent an error email.".to_string(),
            Err(e) => format!(", and subsequently failed to send an error email: {:#?}", e),
        };

        Err(Error {
            message: format!("Could not send email{}", message),
        })
    } else {
        email_result
    }
}
