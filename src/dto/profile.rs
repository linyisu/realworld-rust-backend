use serde::Serialize;

#[derive(Serialize)]
pub struct ProfileResponse {
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub following: bool,
}

#[derive(Serialize)]
pub struct GetResponse {
    pub profile: ProfileResponse,
}

#[derive(Serialize)]
pub struct FollowResponse {
    pub profile: ProfileResponse,
}

#[derive(Serialize)]
pub struct UnfollowResponse {
    pub profile: ProfileResponse,
}
