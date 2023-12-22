use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};

use crate::{configuration::Smtp, email_format::Email, error::Error};

pub fn send_email(email: &Email, configuration: &Smtp) -> Result<(), Error> {
    let email = Message::builder()
        .from(configuration.from.parse().unwrap())
        .reply_to(configuration.reply_to.parse().unwrap())
        .to(configuration.to.parse().unwrap())
        .subject(email.headers.get("Subject").unwrap())
        .body(email.body.to_owned())
        .unwrap();

    let creds = Credentials::new(
        configuration.username.to_owned(),
        configuration.password.to_owned(),
    );

    let mailer = SmtpTransport::relay(configuration.server.as_str())
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error {
            message: format!("Could not send email: {}", e),
        }),
    }
}
