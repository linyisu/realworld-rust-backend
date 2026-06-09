// use sea_orm::entity::prelude::*;
// use serde::Serialize;

// #[derive(Clone, Debug, DeriveEntityModel, Serialize)]
// #[sea_orm(table_name = "articles")]
// pub struct Model {
//     #[sea_orm(primary_key)]
//     id: u32,
//     slug: String,
//     title: String,
//     description: String,
//     body: String,
//     author_id: u32,
//     create_at: String,
//     update_at: String,
// }

// #[derive(Debug, EnumIter, DeriveRelation)]
// pub enum Relation {
//     // 外键 （articles.author_id = users.id）
// }

// impl ActiveModelBehavior for ActiveModel {}
