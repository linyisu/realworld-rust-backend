use sea_orm::entity::prelude::*;
use serde::Serialize;

#[derive(Clone, Debug, DeriveEntityModel, Serialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    #[sea_orm(unique)]
    pub email: String,
    #[sea_orm(unique)]
    pub username: String,
    pub password_hash: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::articles::Entity")]
    Articles,
}

impl Related<super::articles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Articles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
