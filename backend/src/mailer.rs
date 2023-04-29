use lettre::message::Mailbox;
use lettre::{Message, SmtpTransport, Transport};
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::thread;
use std::thread::JoinHandle;

#[derive(Clone)]
pub struct Mailer {
    from: Mailbox,
    public_url: String,
    state: State,
    executor_handler: Arc<JoinHandle<()>>,
    task_sender: SyncSender<Msg>,
}

#[derive(Clone)]
enum State {
    Ready,
    ShuttingDown,
}

impl State {
    fn accepts_messages(&self) -> bool {
        match self {
            State::Ready => true,
            _ => false,
        }
    }
}

enum Msg {
    Stop(Waker),
    Send(Message),
}

pub struct Graceful {
    mailer: Mailer,
}

impl Future for Graceful {
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
    pub fn new(smtp: SmtpTransport, from: Mailbox, public_url: String) -> Self {
        let (tx, rx) = sync_channel::<Msg>(32);

        let executor = {
            thread::spawn(move || {
                while let Ok(task) = rx.recv() {
                    match task {
                        Msg::Send(message) => match smtp.send(&message) {
                            Ok(_) => {}
                            Err(e) => log::error!("Could not send email: {:?}", e),
                        },
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
            state: State::Ready,
            executor_handler: Arc::new(executor),
            task_sender: tx,
        }
    }

    pub fn stop(mut self) -> Graceful {
        self.state = State::ShuttingDown;
        log::info!("Starting graceful shutdown");
        Graceful { mailer: self }
    }

    pub fn send_email_token<S>(&self, token: S, to: Mailbox)
    where
        S: Display,
    {
        self.send(
            Message::builder()
                .from(self.from.clone())
                .to(to)
                .subject("Please validate your email")
                .body(format!(
                    "Please visit {}/email/{}/~validate to complete your registration.",
                    self.public_url, token
                ))
                .unwrap(),
        );
    }

    fn send(&self, message: Message) {
        if !self.state.accepts_messages() {
            log::error!("Shutdown in progress");
        }
        if let Err(e) = self.task_sender.send(Msg::Send(message)) {
            log::error!("Could not send email: {:?}", e);
        }
    }

    pub fn send_email_updated(&self, new_email: &str, to: Mailbox) {
        self.send( Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject("Update of email address")
            .body(format!(
                "An email update validation email was sent to {}. Please follow instruction there to complete the email address update.",
                new_email
            ))
            .unwrap());
    }

    pub fn send_password_updated(&self, to: Mailbox) {
        self.send(
            Message::builder()
                .from(self.from.clone())
                .to(to)
                .subject("Update of password")
                .body("Your password has been updated.".to_string())
                .unwrap(),
        );
    }

    pub fn send_password_recovery<I, T>(&self, id: I, token: T, to: Mailbox)
    where
        I: Display,
        T: Display,
    {
        self.send(
            Message::builder()
                .from(self.from.clone())
                .to(to)
                .subject("Password recovery")
                .body(format!(
                    "Please visit {}/recover-password/{}?token={} to proceed.",
                    self.public_url, id, token
                ))
                .unwrap(),
        );
    }
}
