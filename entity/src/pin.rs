//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "pin")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub bookmark_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::account::Entity",
        from = "Column::UserId",
        to = "super::account::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Account,
    #[sea_orm(
        belongs_to = "super::bookmark::Entity",
        from = "Column::BookmarkId",
        to = "super::bookmark::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Bookmark,
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Account.def()
    }
}

impl Related<super::bookmark::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bookmark.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
