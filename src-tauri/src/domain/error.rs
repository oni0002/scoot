use serde::Serialize;
use thiserror::Error;

/// アプリケーション全体で利用するエラー型
#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Command execution error: {0}")]
    CommandExecution(String),

    #[error("System error: {0}")]
    System(String),
}

/// Tauriがフロントエンドにエラーを返すために必要なSerialize実装
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// 様々な型からAppErrorへの変換をサポートしたい場合のヘルパー
impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::System(err)
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::System(err.to_string())
    }
}
