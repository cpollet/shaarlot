use async_trait::async_trait;
use axum_sessions::async_session::{Session, SessionStore};
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use std::fmt::Debug;
use std::time::Duration;

pub mod session;

#[derive(Clone, Debug)]
pub struct RedisStore {
    connection: MultiplexedConnection,
    ttl: Duration,
}

impl RedisStore {
    pub fn new(connection: MultiplexedConnection, ttl: Duration) -> Self {
        Self { connection, ttl }
    }

    async fn destroy_session_id(&self, session_id: &str) -> axum_sessions::async_session::Result {
        log::trace!("destroying session by id `{}`", session_id);
        let mut connection = self.connection.clone();
        connection.del(session_id).await?;
        Ok(())
    }
}

#[async_trait]
impl SessionStore for RedisStore {
    async fn load_session(
        &self,
        cookie_value: String,
    ) -> axum_sessions::async_session::Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;
        let mut connection = self.connection.clone();
        let json: Option<String> = connection.get(&id).await?;

        log::trace!(
            "loaded session by id `{}`: {}",
            id,
            json.clone().unwrap_or_default()
        );

        if let Some(json) = json {
            let session = serde_json::from_str::<Session>(&json)?;
            let session_id = session.id().to_owned();
            return match session.validate() {
                None => {
                    self.destroy_session_id(&session_id).await?;
                    Ok(None)
                }
                Some(session) => Ok(Some(session)),
            };
        }
        Ok(None)
    }

    async fn store_session(
        &self,
        session: Session,
    ) -> axum_sessions::async_session::Result<Option<String>> {
        let json = serde_json::to_string(&session)?;
        let id = session.id();
        let mut connection = self.connection.clone();
        connection
            .set_ex(id, &json, self.ttl.as_secs() as usize)
            .await?;

        log::trace!("stored session by id `{}`: {}", &id, json);

        session.reset_data_changed();
        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> axum_sessions::async_session::Result {
        self.destroy_session_id(session.id()).await
    }

    async fn clear_store(&self) -> axum_sessions::async_session::Result {
        let mut connection = self.connection.clone();
        let _ = redis::cmd("flushall").query_async(&mut connection).await?;
        Ok(())
    }
}
