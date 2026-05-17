use serde::{Serialize, Deserialize};
use specta::Type;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AppErrorCode {
    NotFound,
    InvalidArg,
    Internal,
    EngineNotReady,
    Unimplemented,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Error)]
#[serde(rename_all = "camelCase")]
#[error("{code:?}: {message}")]
pub struct AppError {
    pub code: AppErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl AppError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self { code: AppErrorCode::NotFound, message: message.into(), details: None }
    }

    pub fn invalid_arg(message: impl Into<String>) -> Self {
        Self { code: AppErrorCode::InvalidArg, message: message.into(), details: None }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self { code: AppErrorCode::Internal, message: message.into(), details: None }
    }

    pub fn unimplemented(message: impl Into<String>) -> Self {
        Self { code: AppErrorCode::Unimplemented, message: message.into(), details: None }
    }
}
