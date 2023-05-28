use chrono::{DateTime, Utc};
use entity::bookmark::ActiveModel;
use entity::bookmark::Model;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ConnectionTrait, DbErr, TryIntoModel};

pub struct Mutation;

impl Mutation {
    #[allow(clippy::too_many_arguments)]
    pub async fn import_bookmark<C>(
        db: &C,
        url: String,
        title: Option<String>,
        description: Option<String>,
        creation_date: DateTime<Utc>,
        update_date: Option<DateTime<Utc>>,
        user_id: i32,
        private: bool,
    ) -> Result<Model, DbErr>
    where
        C: ConnectionTrait,
    {
        ActiveModel {
            id: Default::default(),
            url: Set(url),
            title: Set(title),
            description: Set(description),
            user_id: Set(user_id),
            private: Set(private),
            creation_date: Set(creation_date.into()),
            update_date: Set(update_date.map(|d| d.into())),
        }
        .save(db)
        .await
        .and_then(|m| m.try_into_model())
    }
}
