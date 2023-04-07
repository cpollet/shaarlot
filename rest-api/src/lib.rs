use secrecy::{DebugSecret, SerializableSecret, Zeroize};
use serde::{Deserialize, Serialize};

pub mod application;
pub mod bookmarks;
pub mod error_response;
pub mod password_recoveries;
pub mod sessions;
pub mod tags;
pub mod urls;
pub mod users;
pub mod validate_email;

#[derive(Serialize, Deserialize, Clone)]
pub struct RestPassword(pub String);

impl SerializableSecret for RestPassword {}

impl Zeroize for RestPassword {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}

impl DebugSecret for RestPassword {}

#[cfg(feature = "backend")]
impl<'t> From<&'t RestPassword> for &'t [u8] {
    fn from(value: &'t RestPassword) -> Self {
        value.0.as_bytes()
    }
}

#[cfg(feature = "backend")]
impl<'t> From<&'t RestPassword> for &'t str {
    fn from(value: &'t RestPassword) -> Self {
        value.0.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RestToken(pub String);

impl SerializableSecret for RestToken {}

impl Zeroize for RestToken {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}

impl DebugSecret for RestToken {}

#[cfg(feature = "backend")]
impl<'t> From<&'t RestToken> for &'t [u8] {
    fn from(value: &'t RestToken) -> Self {
        value.0.as_bytes()
    }
}

#[cfg(feature = "backend")]
impl<'t> From<&'t RestToken> for &'t str {
    fn from(value: &'t RestToken) -> Self {
        value.0.as_str()
    }
}
