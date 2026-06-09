use crate::{app_error::AppError, models::follows};

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub async fn is_following(
    db: &DatabaseConnection,
    follower_id: u32,
    followee_id: u32,
) -> Result<bool, AppError> {
    let follow = follows::Entity::find()
        .filter(follows::Column::FollowerId.eq(follower_id))
        .filter(follows::Column::FolloweeId.eq(followee_id))
        .one(db)
        .await?;

    Ok(follow.is_some())
}
