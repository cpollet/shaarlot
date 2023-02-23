use serde::{Deserialize, Serialize};

pub mod bookmarks {
    use super::*;
    use chrono::{DateTime, Utc};

    pub const URL_BOOKMARKS: &str = "/api/bookmarks";
    pub const URL_BOOKMARK: &str = "/api/bookmarks/:id";
    pub const URL_BOOKMARK_QRCODE: &str = "/api/bookmarks/:id/qrcode";

    #[derive(Serialize, Deserialize)]
    pub struct BookmarkResponse {
        pub id: i32,
        pub url: String,
        pub title: Option<String>,
        pub description: Option<String>,
        pub tags: Vec<String>,
        pub creation_date: DateTime<Utc>,
        pub update_date: Option<DateTime<Utc>>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct CreateBookmarkRequest {
        pub url: String,
        pub title: Option<String>,
        pub description: Option<String>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct UpdateBookmarkRequest {
        pub url: String,
        pub title: Option<String>,
        pub description: Option<String>,
    }
}

pub mod urls {
    use super::*;

    pub const URL_URLS: &str = "/api/urls/:url";

    #[derive(Serialize, Deserialize)]
    pub struct UrlResponse {
        pub url: String,
        pub title: Option<String>,
        pub description: Option<String>,
    }
}

pub mod authentication {
    use super::*;
    use secrecy::{DebugSecret, Secret, SerializableSecret, Zeroize};

    pub const URL_USERS: &str = "/api/users";

    #[derive(Serialize, Deserialize, Clone)]
    pub struct RestPassword(pub String);

    impl SerializableSecret for RestPassword {}

    impl Zeroize for RestPassword {
        fn zeroize(&mut self) {
            self.0.zeroize()
        }
    }

    impl DebugSecret for RestPassword {}

    impl<'t> From<&'t RestPassword> for &'t [u8] {
        fn from(value: &'t RestPassword) -> Self {
            value.0.as_bytes()
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct CreateUserRequest {
        pub username: String,
        pub password: Secret<RestPassword>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct UserResponse {
        pub id: i32,
        pub username: String,
    }

    pub mod sessions {
        use super::*;
        use http::StatusCode;

        pub const URL_SESSIONS: &str = "/api/sessions";
        pub const URL_SESSIONS_CURRENT: &str = "/api/sessions/current";

        pub enum CreateSessionResponseCode {
            Success,
            InvalidCredentials,
            Other,
            ServerError,
        }

        #[cfg(feature = "server")]
        impl From<CreateSessionResponseCode> for StatusCode {
            fn from(value: CreateSessionResponseCode) -> Self {
                match value {
                    CreateSessionResponseCode::Success => StatusCode::CREATED,
                    CreateSessionResponseCode::InvalidCredentials => StatusCode::NOT_FOUND,
                    CreateSessionResponseCode::Other => StatusCode::BAD_REQUEST,
                    CreateSessionResponseCode::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
                }
            }
        }

        #[cfg(feature = "server")]
        impl axum::response::IntoResponse for CreateSessionResponseCode {
            fn into_response(self) -> axum::response::Response {
                StatusCode::from(self).into_response()
            }
        }

        #[cfg(feature = "client")]
        impl TryFrom<StatusCode> for CreateSessionResponseCode {
            type Error = ();

            fn try_from(value: StatusCode) -> Result<Self, Self::Error> {
                match value {
                    StatusCode::CREATED => Ok(CreateSessionResponseCode::Success),
                    StatusCode::NOT_FOUND => Ok(CreateSessionResponseCode::InvalidCredentials),
                    StatusCode::BAD_REQUEST => Ok(CreateSessionResponseCode::Other),
                    StatusCode::INTERNAL_SERVER_ERROR => Ok(CreateSessionResponseCode::ServerError),
                    _ => Err(()),
                }
            }
        }

        #[cfg(feature = "client")]
        impl From<gloo_net::http::Response> for CreateSessionResponseCode {
            fn from(value: gloo_net::http::Response) -> Self {
                StatusCode::from_u16(value.status())
                    .map_err(|_| ())
                    .and_then(StatusCode::try_into)
                    .unwrap_or(CreateSessionResponseCode::Other)
            }
        }

        #[derive(Serialize, Deserialize)]
        pub struct CreateSessionRequest {
            pub username: String,
            pub password: Secret<RestPassword>,
        }

        #[derive(Serialize, Deserialize)]
        pub struct SessionResponse {
            pub username: String,
        }
    }
}
