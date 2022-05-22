
use axum::{
    extract::{Extension, FromRequest, Json, Query, RequestParts, rejection::ExtensionRejection},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tera::{Context, Tera};
use tera;
use axum::async_trait;

pub struct Render {
    tera: Arc<Tera>,
    name: String,
}

#[derive(Clone)]
pub struct Templates(Arc<Tera>);

#[async_trait]
impl<B> FromRequest<B> for Templates
where
    B: Send,
{
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match Extension::<Templates>::from_request(req).await {
            Ok(Extension(t)) => Ok(t),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

impl Templates {
    pub fn new(path: &str) -> Result<Templates, tera::Error> {
        let mut tera = Tera::new(path)?;
        tera.autoescape_on(vec!["html"]);
        //        tera.register_filter("do_nothing", do_nothing_filter);
        Ok(Templates(Arc::new(tera)))
    }

    pub fn render(self: &Templates, name: &str) -> Render {
        Render {
            name: name.to_string(),
            tera: self.0.clone(),
        }
    }
}

impl IntoResponse for Render {
    fn into_response(self) -> Response {
        match self.tera.render(&self.name, &Context::new()) {
            Ok(html) => (StatusCode::OK, [(header::CONTENT_TYPE, "text/html")], html).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)).into_response()
        }
    }
}
