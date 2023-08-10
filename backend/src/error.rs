use std::fmt;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use ulid::Ulid;

#[derive(Debug)]
pub struct Error {
    pub id: Ulid,
    pub inner: anyhow::Error,
    pub status_code: StatusCode,
}

#[derive(Serialize)]
struct ResponseError {
    id: Ulid,
    error: String,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let resp = ResponseError {
            id: self.id,
            error: self.inner.to_string(),
        };
        if self.status_code.is_server_error() {
            tracing::error!(id = %self.id, error = ?self.inner, "response error");
        }
        (self.status_code, Json(resp)).into_response()
    }
}

impl<E> From<E> for Error
where
    anyhow::Error: From<E>,
{
    fn from(value: E) -> Self {
        Self {
            id: Ulid::new(),
            inner: value.into(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait AddContext<T> {
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static;

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T> AddContext<T> for Result<T> {
    fn context<C>(self, context: C) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        match self {
            Err(mut error) => {
                error.inner = error.inner.context(context);
                Err(error)
            }
            ok => ok,
        }
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        match self {
            Err(mut error) => {
                error.inner = error.inner.context(f());
                Err(error)
            }
            ok => ok,
        }
    }
}

pub trait Context<T> {
    fn context<C>(self, context: C, status_code: StatusCode) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static;

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> (C, StatusCode);

    fn context_internal_server_error<C>(self, context: C) -> Result<T>
    where
        Self: Sized,
        C: fmt::Display + Send + Sync + 'static,
    {
        self.context(context, StatusCode::INTERNAL_SERVER_ERROR)
    }

    fn context_bad_request<C>(self, context: C) -> Result<T>
    where
        Self: Sized,
        C: fmt::Display + Send + Sync + 'static,
    {
        self.context(context, StatusCode::BAD_REQUEST)
    }

    fn context_unauthorized<C>(self, context: C) -> Result<T>
    where
        Self: Sized,
        C: fmt::Display + Send + Sync + 'static,
    {
        self.context(context, StatusCode::UNAUTHORIZED)
    }
}

impl<T, E> Context<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context<C>(self, context: C, status_code: StatusCode) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => {
                let id = Ulid::new();
                let inner = anyhow::Error::new(error).context(context);
                Err(Error {
                    id,
                    inner,
                    status_code,
                })
            }
        }
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> (C, StatusCode),
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => {
                let (context, status_code) = f();
                let id = Ulid::new();
                let inner = anyhow::Error::new(error).context(context);
                Err(Error {
                    id,
                    inner,
                    status_code,
                })
            }
        }
    }
}

impl<T> Context<T> for std::option::Option<T> {
    fn context<C>(self, context: C, status_code: StatusCode) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
    {
        match self {
            Some(ok) => Ok(ok),
            None => {
                let id = Ulid::new();
                let inner = anyhow::format_err!("{}", context);
                Err(Error {
                    id,
                    inner,
                    status_code,
                })
            }
        }
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + Send + Sync + 'static,
        F: FnOnce() -> (C, StatusCode),
    {
        match self {
            Some(ok) => Ok(ok),
            None => {
                let (context, status_code) = f();
                let id = Ulid::new();
                let inner = anyhow::format_err!("{}", context);
                Err(Error {
                    id,
                    inner,
                    status_code,
                })
            }
        }
    }
}

#[macro_export]
macro_rules! format_err {
    ($status_code:ident, $msg:literal $(,)?) => {
        $crate::error::Error {
            id: ::ulid::Ulid::new(),
            status_code: ::axum::http::StatusCode::$status_code,
            inner: anyhow::format_err!($msg),
        }
    };
    ($status_code:expr, $fmt:expr, $($arg:tt)*) => {
        $crate::error::Error {
            id: ::ulid::Ulid::new(),
            status_code: ::axum::http::StatusCode::$status_code,
            inner: anyhow::format_err!($fmt, $($arg,)*),
        }
    };
}
