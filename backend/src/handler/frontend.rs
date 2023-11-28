use axum::response::{Html, IntoResponse};

pub mod assets;

static INDEX_HTML: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../frontend/dist/index.html"
));

pub async fn get_index() -> Html<&'static [u8]> {
    Html(INDEX_HTML)
}

pub enum RespOrFrontend<T> {
    Resp(T),
    Frontend,
}

impl<T> IntoResponse for RespOrFrontend<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Resp(resp) => resp.into_response(),
            Self::Frontend => Html(INDEX_HTML).into_response(),
        }
    }
}
