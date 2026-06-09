use crate::app_error::AppError;

use percent_encoding::percent_decode_str;

pub fn slugify(title: &str) -> String {
    title
        .trim()
        .to_lowercase()
        .replace(" ", "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

pub fn decode_slug(slug: &str) -> Result<String, AppError> {
    percent_decode_str(slug)
        .decode_utf8()
        .map(|s| s.to_string())
        .map_err(|_| AppError::BadRequest)
}
