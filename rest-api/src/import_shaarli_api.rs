use secrecy::{DebugSecret, Secret, SerializableSecret, Zeroize};
use serde::{Deserialize, Serialize};

pub const URL_SHAARLI_IMPORT_API: &str = "/api/shaarli-import-api";

#[derive(Serialize, Deserialize)]
pub struct ShaarliImportApiRequest {
    pub url: String,
    // todo keep the secret key in the browser only
    pub key: Secret<ShaarliApiKey>,
}

pub enum ShaarliImportApiResult {
    Success,
    Forbidden,
    ShaarliError,
    NotImplemented,
    ServerError,

    #[cfg(feature = "frontend")]
    BrowserError,
    #[cfg(feature = "frontend")]
    DeserializationError,
}

#[cfg(feature = "frontend")]
impl ShaarliImportApiResult {
    pub async fn from(value: Result<gloo_net::http::Response, gloo_net::Error>) -> Option<Self> {
        match value {
            Err(_) => Some(ShaarliImportApiResult::BrowserError),
            Ok(response) => match response.status() {
                200 => Some(ShaarliImportApiResult::Success),
                400 => Some(ShaarliImportApiResult::ShaarliError),
                403 => Some(ShaarliImportApiResult::Forbidden),
                500 => Some(ShaarliImportApiResult::ServerError),
                501 => Some(ShaarliImportApiResult::NotImplemented),
                _ => {
                    // todo add log
                    None
                }
            },
        }
    }
}

#[cfg(feature = "backend")]
impl axum::response::IntoResponse for ShaarliImportApiResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            ShaarliImportApiResult::Success => http::StatusCode::OK.into_response(),
            ShaarliImportApiResult::ShaarliError => http::StatusCode::BAD_REQUEST.into_response(),
            ShaarliImportApiResult::Forbidden => http::StatusCode::FORBIDDEN.into_response(),
            ShaarliImportApiResult::NotImplemented => {
                http::StatusCode::NOT_IMPLEMENTED.into_response()
            }
            ShaarliImportApiResult::ServerError => {
                http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            _ => panic!(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ShaarliApiKey(pub String);

impl SerializableSecret for ShaarliApiKey {}

impl Zeroize for ShaarliApiKey {
    fn zeroize(&mut self) {
        self.0.zeroize()
    }
}

impl DebugSecret for ShaarliApiKey {}

#[cfg(feature = "backend")]
impl<'t> From<&'t ShaarliApiKey> for &'t [u8] {
    fn from(value: &'t ShaarliApiKey) -> Self {
        value.0.as_bytes()
    }
}

#[cfg(feature = "backend")]
impl<'t> From<&'t ShaarliApiKey> for &'t str {
    fn from(value: &'t ShaarliApiKey) -> Self {
        value.0.as_str()
    }
}
