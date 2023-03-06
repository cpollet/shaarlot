use lettre::message::Mailbox;
use lettre::{Message, SmtpTransport, Transport};

#[derive(Clone)]
pub struct Mailer {
    pub smtp: SmtpTransport,
    pub from: Mailbox,
    pub public_url: String,
}

impl Mailer {
    pub fn send_email_token(&self, token: String, to: Mailbox) {
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
}
