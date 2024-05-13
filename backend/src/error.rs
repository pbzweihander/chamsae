use std::fmt;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use tracing_error::SpanTrace;
use ulid::Ulid;

use crate::config::CONFIG;

#[derive(Debug)]
pub struct Error {
    pub id: Ulid,
    pub inner: anyhow::Error,
    pub status_code: StatusCode,
    pub context: SpanTrace,
}

impl Error {
    pub fn new<M>(status_code: StatusCode, message: M) -> Self
    where
        M: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        Self::from_anyhow(status_code, anyhow::Error::msg(message))
    }

    pub fn from_anyhow(status_code: StatusCode, inner: anyhow::Error) -> Self {
        let id = Ulid::new();
        let context = SpanTrace::capture();
        Self {
            id,
            inner,
            status_code,
            context,
        }
    }
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
            tracing::error!(
                "responding server error, id: {}\n{:?}\nContext:\n{}",
                self.id,
                self.inner,
                self.context
            );
        } else if CONFIG.debug {
            tracing::warn!(
                "responding client error, id: {}\n{:?}\nContext\n{}",
                self.id,
                self.inner,
                self.context
            );
        } else {
            tracing::debug!(
                "responding client error, id: {}\n{:?}\nContext\n{}",
                self.id,
                self.inner,
                self.context
            );
        }
        (self.status_code, Json(resp)).into_response()
    }
}

impl<E> From<E> for Error
where
    anyhow::Error: From<E>,
{
    fn from(value: E) -> Self {
        Self::from_anyhow(StatusCode::INTERNAL_SERVER_ERROR, value.into())
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
        C: fmt::Display + fmt::Debug + Send + Sync + 'static;

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + fmt::Debug + Send + Sync + 'static,
        F: FnOnce() -> (C, StatusCode);

    fn context_bad_request<C>(self, context: C) -> Result<T>
    where
        Self: Sized,
        C: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        self.context(context, StatusCode::BAD_REQUEST)
    }

    fn context_unauthorized<C>(self, context: C) -> Result<T>
    where
        Self: Sized,
        C: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        self.context(context, StatusCode::UNAUTHORIZED)
    }

    fn context_not_found<C>(self, context: C) -> Result<T>
    where
        Self: Sized,
        C: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        self.context(context, StatusCode::NOT_FOUND)
    }

    fn context_internal_server_error<C>(self, context: C) -> Result<T>
    where
        Self: Sized,
        C: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        self.context(context, StatusCode::INTERNAL_SERVER_ERROR)
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
                let inner = anyhow::Error::new(error).context(context);
                Err(Error::from_anyhow(status_code, inner))
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
                let inner = anyhow::Error::new(error).context(context);
                Err(Error::from_anyhow(status_code, inner))
            }
        }
    }
}

impl<T> Context<T> for std::option::Option<T> {
    fn context<C>(self, context: C, status_code: StatusCode) -> Result<T>
    where
        C: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        match self {
            Some(ok) => Ok(ok),
            None => Err(Error::new(status_code, context)),
        }
    }

    fn with_context<C, F>(self, f: F) -> Result<T>
    where
        C: fmt::Display + fmt::Debug + Send + Sync + 'static,
        F: FnOnce() -> (C, StatusCode),
    {
        match self {
            Some(ok) => Ok(ok),
            None => {
                let (context, status_code) = f();
                Err(Error::new(status_code, context))
            }
        }
    }
}

#[macro_export]
macro_rules! format_err {
    ($status_code:ident, $msg:literal $(,)?) => {
        $crate::error::Error::new(::axum::http::StatusCode::$status_code, $msg)
    };
    ($status_code:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::error::Error::from_anyhow(::axum::http::StatusCode::$status_code, ::anyhow::format_err!($fmt, $($arg)*))
    };
}
