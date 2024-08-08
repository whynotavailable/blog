use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug)]
pub struct AppError {
    pub status_code: StatusCode,
    pub message: String,
}

impl AppError {
    pub fn new(message: String) -> AppError {
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message,
        }
    }

    pub fn status(status_code: StatusCode) -> AppError {
        AppError {
            status_code,
            message: "".to_string(),
        }
    }

    pub fn not_found(message: &str) -> AppError {
        AppError {
            status_code: StatusCode::NOT_FOUND,
            message: message.to_string(),
        }
    }

    pub fn from<T: std::error::Error>(e: T) -> AppError {
        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: e.to_string(),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status_code, self.message)
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code, self.message).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error> + std::fmt::Display,
{
    fn from(err: E) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("{}", err),
        }
    }
}
