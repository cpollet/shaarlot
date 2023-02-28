use secrecy::{DebugSecret, SerializableSecret, Zeroize};
use serde::{Deserialize, Serialize};

pub mod bookmarks;
pub mod error_response;
pub mod sessions;
pub mod urls;
pub mod users;

#[cfg(all(feature = "frontend", feature = "backend"))]
compile_error!("feature \"frontend\" and feature \"backend\" cannot be enabled at the same time");

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
