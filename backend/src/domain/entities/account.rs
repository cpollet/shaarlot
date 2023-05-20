use chrono::{DateTime, FixedOffset, Utc};
use uuid::Uuid;

pub struct Account {
    // todo remove pub
    pub id: i32,
    pub username: String,
    pub password: Password,
    pub creation_date: DateTime<Utc>,
    pub email: Option<String>,
    pub next_email: Option<NextEmail>,
}

pub enum Password {
    Keep,
    Change(String),
}

pub struct NextEmail {
    email: String,
    token: Uuid,
    token_generation_date: DateTime<Utc>,
}

impl NextEmail {
    fn token_expired(&self) -> bool {
        let now = DateTime::<FixedOffset>::from(Utc::now());
        let duration = now.signed_duration_since(self.token_generation_date);

        duration.num_minutes() > 60
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn token(&self) -> Uuid {
        self.token
    }

    pub fn token_generation_date(&self) -> &DateTime<Utc> {
        &self.token_generation_date
    }
}

impl NextEmail {
    pub fn new(email: String, token: Uuid, token_generation_date: DateTime<Utc>) -> Self {
        Self {
            email,
            token,
            token_generation_date,
        }
    }
}

pub enum ValidateEmailError {
    InvalidToken,
}

impl Account {
    pub fn validate_email(mut self) -> Result<Self, ValidateEmailError> {
        match self.next_email {
            None => Ok(self),
            Some(next_email) if next_email.token_expired() => Err(ValidateEmailError::InvalidToken),
            Some(next_email) => {
                self.email = Some(next_email.email);
                self.next_email = None;
                Ok(self)
            }
        }
    }
}
