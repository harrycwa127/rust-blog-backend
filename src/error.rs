use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use sea_orm::{DbErr, RuntimeErr};
use tracing::{error, warn};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("內部伺服器錯誤: {0}")]
    InternalServerError(String),
    #[error("找不到資源: {0}")]
    NotFound(String),
    #[error("請求無效: {0}")]
    BadRequest(String),
    #[error("未授權: {0}")]
    Unauthorized(String),
    #[error("參數驗證失敗: {0}")]
    ValidationError(String),
    #[error("資源衝突: {0}")]
    ConflictError(String),
    #[error("禁止存取: {0}")]
    Forbidden(String),
}

impl AppError {
    #[inline]
    fn status_and_client_msg(&self) -> (StatusCode, &'static str) {
        match self {
            AppError::InternalServerError(_)   => (StatusCode::INTERNAL_SERVER_ERROR, "內部伺服器錯誤"),
            AppError::NotFound(_)           => (StatusCode::NOT_FOUND,              "找不到資源"),
            AppError::BadRequest(_)         => (StatusCode::BAD_REQUEST,            "請求無效"),
            AppError::Unauthorized(_)          => (StatusCode::UNAUTHORIZED,           "未授權"),
            AppError::ValidationError(_)    => (StatusCode::UNPROCESSABLE_ENTITY,   "參數驗證失敗"),
            AppError::ConflictError(_)      => (StatusCode::CONFLICT,               "資源衝突"),
            AppError::Forbidden(_)             => (StatusCode::FORBIDDEN,              "禁止存取"),

        }
    }

    #[inline]
    fn log(&self) {
        match self {
            AppError::InternalServerError(msg) => error!("內部伺服器錯誤: {msg}"),
            AppError::NotFound(msg)       => warn!("資源未找到: {msg}"),
            AppError::BadRequest(msg)     => warn!("請求無效: {msg}"),
            AppError::Unauthorized(msg)        => warn!("未授權的存取嘗試: {msg}"),
            AppError::ValidationError(msg)=> warn!("驗證失敗: {msg}"),
            AppError::ConflictError(msg)  => warn!("資源衝突: {msg}"),
            AppError::Forbidden(msg)             => warn!("禁止存取的嘗試: {msg}"),
        }
    }
}

/// 從 SeaORM 錯誤取出 Postgres SQLSTATE（需要 sqlx 依賴）
#[inline]
fn sqlstate_code(err: &DbErr) -> Option<String> {
    if let DbErr::Exec(rt) | DbErr::Query(rt) = err {
        if let RuntimeErr::SqlxError(sqlx_err) = rt {
            return sqlx_err
                .as_database_error()
                .and_then(|db_err| db_err.code().map(|c| c.as_ref().to_owned()));
        }
    }
    None
}

impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        if matches!(err, DbErr::RecordNotFound(_)) {
            return AppError::NotFound("資料不存在".into());
        }

        if let Some(code) = sqlstate_code(&err).as_deref() {
            return match code {
                "23505" => AppError::ConflictError("唯一鍵衝突（例如 slug 已存在）".into()),
                "23503" => AppError::BadRequest("外鍵約束失敗".into()),
                "23502" => AppError::BadRequest("必填欄位為空 (NOT NULL)".into()),
                "23514" => AppError::BadRequest("檢查條件不符合 (CHECK)".into()),
                "22001" => AppError::BadRequest("字串超過長度限制".into()),
                _       => {
                    error!("未處理的 SQLSTATE: {code}");
                    AppError::InternalServerError(format!("未處理的 SQLSTATE: {code}"))
                }
            };
        }

        // 後備字串偵測（不同 DB/語系可能略有差異）
        let s = err.to_string();
        if s.contains("duplicate key value")                  { return AppError::ConflictError("唯一鍵衝突（可能是 slug 重複）".into()); }
        if s.contains("violates foreign key constraint")      { return AppError::BadRequest("外鍵約束失敗".into()); }
        if s.contains("null value in column")                 { return AppError::BadRequest("必填欄位為空 (NOT NULL)".into()); }
        if s.contains("value too long for type") || s.contains("string data right truncation") {
            return AppError::BadRequest("字串超過長度限制".into());
        }

        error!("資料庫錯誤: {err:#}");
        AppError::InternalServerError(format!("資料庫錯誤 {err:#}"))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.log();
        let (status, client_msg) = self.status_and_client_msg();

        (status, Json(json!({
            "error": client_msg,
            "status": status.as_u16(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))).into_response()
    }
}