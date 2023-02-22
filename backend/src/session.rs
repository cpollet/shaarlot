use crate::rest::error_response::ErrorResponse;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_sessions::SessionHandle;

#[derive(Clone)]
pub struct UserInfo {
    pub id: i32,
}

pub struct Session;

impl Session {
    pub async fn required<B>(
        mut request: Request<B>,
        next: axum::middleware::Next<B>,
    ) -> impl IntoResponse {
        if let Some(session_handle) = request.extensions().get::<SessionHandle>() {
            let uid = { session_handle.read().await.get::<i32>("USERID") };
            if let Some(id) = uid {
                log::info!("Continue with user {}", id);
                request.extensions_mut().insert(UserInfo { id });
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
            let uid = { session_handle.read().await.get::<i32>("USERID") };
            if let Some(id) = uid {
                log::info!("Continue with user {}", id);
                request.extensions_mut().insert(Some(UserInfo { id }));
            } else {
                request.extensions_mut().insert::<Option<UserInfo>>(None);
            }
        }
        next.run(request).await
    }
}
