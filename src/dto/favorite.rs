use serde::Serialize;

#[derive(Serialize)]
pub struct FavoriteResponse {
    pub article: super::article::ArticleResponse,
}

#[derive(Serialize)]
pub struct UnfavoriteResponse {
    pub article: super::article::ArticleResponse,
}
