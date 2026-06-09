use serde::Serialize;

#[derive(Serialize)]
pub struct ArticleResponse {
    pub favorited: bool,
    pub favorites_count: u32,
}

#[derive(Serialize)]
pub struct FavoriteResponse {
    pub article: ArticleResponse,
}

#[derive(Serialize)]
pub struct UnfavoriteResponse {
    pub article: ArticleResponse,
}
