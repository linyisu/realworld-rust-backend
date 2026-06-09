use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SignupUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub user: SignupUser,
}

#[derive(Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub user: LoginUser,
}

#[derive(Deserialize)]
pub struct UpdateUser {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateRequest {
    pub user: UpdateUser,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub email: String,
    pub token: String,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Serialize)]
pub struct SignupResponse {
    pub user: UserResponse,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub user: UserResponse,
}

#[derive(Serialize)]
pub struct GetResponse {
    pub user: UserResponse,
}

#[derive(Serialize)]
pub struct UpdateResponse {
    pub user: UserResponse,
}
