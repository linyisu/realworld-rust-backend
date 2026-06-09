use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct UpdateUser {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(deserialize_with = "deserialize_nullable_field")]
    pub bio: Option<Option<String>>,
    #[serde(deserialize_with = "deserialize_nullable_field")]
    pub image: Option<Option<String>>,
}

impl Default for UpdateUser {
    fn default() -> Self {
        Self {
            email: None,
            username: None,
            password: None,
            bio: None,
            image: None,
        }
    }
}

fn deserialize_nullable_field<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<Option<String>>::deserialize(deserializer)?;
    Ok(Some(opt.unwrap_or(None)))
}

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
