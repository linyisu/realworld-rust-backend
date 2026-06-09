use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "article_tags")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub article_id: u32,

    #[sea_orm(primary_key, auto_increment = false)]
    pub tag_id: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::articles::Entity",
        from = "Column::ArticleId",
        to = "super::articles::Column::Id"
    )]
    Article,

    #[sea_orm(
        belongs_to = "super::tags::Entity",
        from = "Column::TagId",
        to = "super::tags::Column::Id"
    )]
    Tag,
}

impl ActiveModelBehavior for ActiveModel {}
