// use crate::middleware::auth::AuthUser;

// use axum::{
//     Json,
//     extract::{Path, State},
// };
// use sea_orm::{
//     ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
// };

// async fn get_profile(
//     auth_user: AuthUser,
//     State(db): State<DatabaseConnection>,
//     Path(username): Path<String>,
// ) -> Result<Json<UpdateResponse>, AppError> {
// }
