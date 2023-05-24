use lettre::message::Mailbox;
use lettre::{Address, Message, SmtpTransport, Transport};
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::thread::JoinHandle;

#[derive(Clone)]
pub struct Mailer {
    from: Mailbox,
    public_url: String,
    state: Arc<RwLock<State>>,
    executor_handler: Arc<JoinHandle<()>>,
    task_sender: SyncSender<Msg>,
}

#[derive(Clone, Debug)]
enum State {
    Ready,
    ShuttingDown,
}

impl State {
    fn accepts_messages(&self) -> bool {
        matches!(self, State::Ready)
    }
}

enum Msg {
    Stop(Waker),
    Send(Email),
}

#[derive(Debug)]
pub struct Email {
    from: Mailbox,
    to: Address,
    subject: String,
    body: String,
}

pub struct Graceful<'a> {
    mailer: &'a Mailer,
}

impl<'a> Future for Graceful<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.mailer.executor_handler.is_finished() {
            return Poll::Ready(());
        }

        let _ = self.mailer.task_sender.send(Msg::Stop(cx.waker().clone()));
        Poll::Pending
    }
}

impl Mailer {
    pub fn new(sender: Box<dyn Sendmail + Send>, from: Mailbox, public_url: String) -> Self {
        let (tx, rx) = sync_channel::<Msg>(32);

        let executor = {
            thread::spawn(move || {
                while let Ok(task) = rx.recv() {
                    match task {
                        Msg::Send(email) => {
                            sender.send(email);
                        }
                        Msg::Stop(waker) => {
                            waker.wake();
                            return;
                        }
                    }
                }
            })
        };

        Self {
            from,
            public_url,
            state: Arc::new(RwLock::new(State::Ready)),
            executor_handler: Arc::new(executor),
            task_sender: tx,
        }
    }

    pub fn stop(&self) -> Graceful {
        {
            let mut s = self.state.write().unwrap();
            *s = State::ShuttingDown;
        }
        log::info!(
            "Starting graceful shutdown (state={:?})",
            self.state.read().unwrap()
        );
        Graceful { mailer: self }
    }

    pub fn send_email_token<S>(&self, token: S, to: Address)
    where
        S: Display,
    {
        self.send(Email {
            from: self.from.clone(),
            to,
            subject: "Please validate your email".to_string(),
            body: format!(
                "Please visit {}/email/{}/~validate to complete your registration.",
                self.public_url, token
            ),
        });
    }

    fn send(&self, email: Email) {
        if !self.state.read().unwrap().accepts_messages() {
            log::error!("Shutdown in progress");
        }
        if let Err(e) = self.task_sender.send(Msg::Send(email)) {
            log::error!("Could not send email: {:?}", e);
        }
    }

    pub fn send_email_updated(&self, to: Address, new_email: Address) {
        self.send(
            Email{
                from: self.from.clone(),
                to,
                subject: "Update of email address".to_string(),
                body:format!(
                    "An email update validation email was sent to {}. Please follow instruction there to complete the email address update.",
                    new_email
                ),
            }
        );
    }

    pub fn send_password_updated(&self, to: Address) {
        self.send(Email {
            from: self.from.clone(),
            to,
            subject: "Update of password".to_string(),
            body: "Your password has been updated.".to_string(),
        });
    }

    pub fn send_password_recovery<I, T>(&self, id: I, token: T, to: Address)
    where
        I: Display,
        T: Display,
    {
        self.send(Email {
            from: self.from.clone(),
            to,
            subject: "Password recovery".to_string(),
            body: format!(
                "Please visit {}/recover-password/{}?token={} to proceed.",
                self.public_url, id, token
            ),
        });
    }
}

pub trait Sendmail {
    fn send(&self, email: Email);
}

pub struct MailSender {
    pub smtp: SmtpTransport,
}

impl Sendmail for MailSender {
    fn send(&self, email: Email) {
        log::info!("send email");
        match self.smtp.send(
            &Message::builder()
                .from(email.from)
                .to(Mailbox::from_str(email.to.as_ref()).expect("cannot be invalid"))
                .subject(email.subject)
                .body(email.body)
                .unwrap(),
        ) {
            Ok(_) => {}
            Err(e) => log::error!("Could not send email: {:?}", e),
        }
    }
}

pub struct LogSender {}

impl Sendmail for LogSender {
    fn send(&self, email: Email) {
        log::info!("{:?}", email);
    }
}
