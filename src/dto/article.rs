use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateArticle {
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct CreateRequest {
    pub article: CreateArticle,
}

#[derive(Deserialize)]
pub struct UpdateArticle {
    pub title: Option<String>,
    pub description: Option<String>,
    pub body: Option<String>,
    pub tag_list: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct UpdateRequest {
    pub article: UpdateArticle,
}

#[derive(Serialize)]
pub struct ArticleResponse {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub favorited: bool,
    pub favorites_count: u32,
    pub author: super::profile::ProfileResponse,
}

#[derive(Serialize)]
pub struct CreateResponse {
    pub article: ArticleResponse,
}

#[derive(Serialize)]
pub struct GetResponse {
    pub article: ArticleResponse,
}

#[derive(Serialize)]
pub struct UpdateResponse {
    pub article: ArticleResponse,
}

#[derive(Serialize)]
pub struct ListResponse {
    pub articles: Vec<ArticleResponse>,
    pub articles_count: usize,
}
