use anyhow::anyhow;
use salvo::{http::StatusCode, writing::Json};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("参数解析错误: {0}")]
    SalvoParseError(#[from] salvo::http::ParseError),
    #[error("参数解析错误")]
    ParamParseError,
    #[error("没有权限")]
    PermissionDenied,
    #[error("没有登录")]
    Unauthorized,
    #[error("数据库错误")]
    DatabaseError(#[from] sqlx::Error),
    #[error("请求超时")]
    TimeoutError,
}

pub struct Success(serde_json::Value);
impl<T: serde::Serialize> From<T> for Success {
    fn from(value: T) -> Self {
        Success(serde_json::json!({
            "code": 200,
            "data": value,
            "msg": "请求成功"
        }))
    }
}
impl salvo::Scribe for Success {
    fn render(self, res: &mut salvo::Response) {
        res.stuff(StatusCode::OK, Json(self.0));
    }
}
impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Anyhow(anyhow!(s.to_string()))
    }
}
impl salvo::Scribe for AppError {
    fn render(self, res: &mut salvo::Response) {
        tracing::error!("{:#?}", self);
        match self {
            AppError::Anyhow(_) | AppError::DatabaseError(_) => res.stuff(
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "code": 500,
                    "data": null,
                    "msg": format!("{}", self)
                })),
            ),
            AppError::SalvoParseError(_) | AppError::ParamParseError => res.stuff(
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "code": 400,
                    "data": null,
                    "msg": "参数解析错误"
                })),
            ),
            AppError::PermissionDenied => res.stuff(
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "code": 403,
                    "data": null,
                    "msg": "没有权限"
                })),
            ),
            AppError::Unauthorized => res.stuff(
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "code": 401,
                    "data": null,
                    "msg": "未登录"
                })),
            ),
            AppError::TimeoutError => res.stuff(
                StatusCode::REQUEST_TIMEOUT,
                Json(serde_json::json!({
                    "code": 408,
                    "data": null,
                    "msg": "请求超时"
                })),
            ),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
pub type RouterResult = AppResult<Success>;
