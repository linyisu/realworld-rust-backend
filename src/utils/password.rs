use crate::app_error::AppError;
use bcrypt::{DEFAULT_COST, hash, verify};

pub fn hash_password(password: &str) -> Result<String, AppError> {
    hash(password, DEFAULT_COST).map_err(|e| {
        eprintln!("密码加密失败: {}", e);
        AppError::InternalServerError
    })
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    verify(password, hash).map_err(|e| {
        eprintln!("密码验证失败: {}", e);
        AppError::InternalServerError
    })
}
