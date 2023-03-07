use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_sessions::SessionHandle;
use rest_api::error_response::ErrorResponse;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub const SESSION_KEY_USER_INFO: &str = "USER_INFO";

#[derive(Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
}

impl Display for UserInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[id:{}, username:{}]", self.id, self.username)
    }
}

pub struct SessionHint;

impl SessionHint {
    pub async fn required<B>(
        mut request: Request<B>,
        next: axum::middleware::Next<B>,
    ) -> impl IntoResponse {
        if let Some(session_handle) = request.extensions().get::<SessionHandle>() {
            let user_info = {
                session_handle
                    .read()
                    .await
                    .get::<UserInfo>(SESSION_KEY_USER_INFO)
            };
            if let Some(user_info) = user_info {
                log::info!("Requested session configured with user {}", user_info);
                request.extensions_mut().insert(user_info);
                return next.run(request).await;
            }
        } else {
            log::warn!("No session found");
        }

        (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse::new(
                "FORBIDDEN",
                "You're not allowed to access this resource",
            )),
        )
            .into_response()
    }

    pub async fn supported<B>(
        mut request: Request<B>,
        next: axum::middleware::Next<B>,
    ) -> Response {
        if let Some(session_handle) = request.extensions().get::<SessionHandle>() {
            let user_info = {
                session_handle
                    .read()
                    .await
                    .get::<UserInfo>(SESSION_KEY_USER_INFO)
            };
            if let Some(user_info) = user_info {
                log::info!("Supported session configured with user {}", user_info);
                request.extensions_mut().insert(Some(user_info));
            } else {
                request.extensions_mut().insert::<Option<UserInfo>>(None);
            }
        }
        next.run(request).await
    }
}
