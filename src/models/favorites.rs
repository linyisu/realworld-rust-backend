use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "favorites")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: u32,

    #[sea_orm(primary_key, auto_increment = false)]
    pub article_id: u32,

    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,

    #[sea_orm(
        belongs_to = "super::articles::Entity",
        from = "Column::ArticleId",
        to = "super::articles::Column::Id"
    )]
    Article,
}

impl ActiveModelBehavior for ActiveModel {}
