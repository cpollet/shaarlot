use lettre::message::Mailbox;
use lettre::{Message, SmtpTransport, Transport};
use std::fmt::Display;

#[derive(Clone)]
pub struct Mailer {
    pub smtp: SmtpTransport,
    pub from: Mailbox,
    pub public_url: String,
}

// todo make it run in a different thread

impl Mailer {
    pub fn send_email_token<S>(&self, token: S, to: Mailbox)
    where
        S: Display,
    {
        let email = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject("Please validate your email")
            .body(format!(
                "Please visit {}/email/{}/~validate to complete your registration.",
                self.public_url, token
            ))
            .unwrap();
        match self.smtp.send(&email) {
            Ok(_) => {}
            Err(e) => log::error!("Could not send email: {:?}", e),
        }
    }

    pub fn send_email_updated(&self, new_email: &str, to: Mailbox) {
        let email = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject("Update of email address")
            .body(format!(
                "An email update validation email was sent to {}. Please follow instruction there to complete the email address update.",
                new_email
            ))
            .unwrap();
        match self.smtp.send(&email) {
            Ok(_) => {}
            Err(e) => log::error!("Could not send email: {:?}", e),
        }
    }

    pub fn send_password_updated(&self, to: Mailbox) {
        let email = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject("Update of password")
            .body("Your password has been updated.".to_string())
            .unwrap();
        match self.smtp.send(&email) {
            Ok(_) => {}
            Err(e) => log::error!("Could not send email: {:?}", e),
        }
    }

    pub fn send_password_recovery<I, T>(&self, id: I, token: T, to: Mailbox)
    where
        I: Display,
        T: Display,
    {
        let email = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject("Password recovery")
            .body(format!(
                "Please visit {}/recover-password/{}?token={} to proceed.",
                self.public_url, id, token
            ))
            .unwrap();

        match self.smtp.send(&email) {
            Ok(_) => {}
            Err(e) => log::error!("Could not send email: {:?}", e),
        }
    }
}
