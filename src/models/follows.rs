use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "follows")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub follower_id: u32,

    #[sea_orm(primary_key, auto_increment = false)]
    pub followee_id: u32,

    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::FollowerId",
        to = "super::users::Column::Id"
    )]
    Follower,

    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::FolloweeId",
        to = "super::users::Column::Id"
    )]
    Followee,
}

impl ActiveModelBehavior for ActiveModel {}
