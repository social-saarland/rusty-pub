use tokio;

use std::error::Error;

use axum::{
    extract::Extension,
    response::IntoResponse,
    routing::get,
    Router,
};

use dotenv::dotenv;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

pub mod activitypub;
pub mod user;
pub mod webfinger;

mod auth;
mod db;
mod templating;

use templating::Templates;
use user::User;

use db::Db;

async fn index(tmpl: Templates) -> impl IntoResponse {
    tmpl.render("index.html")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let templates = match Templates::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => panic!("Failed to load templates: {}", e),
    };

    let db = match Db::from_env().await {
        Ok(db) => db,
        Err(e) => panic!("Could not connect to database: {}", e),
    };

    let app = Router::new()
        //  just some fake root to check if the app is running
        .route("/", get(index))
        //  Tthis handler is called by Mastodon if you search for a user.
        //  See the Webfinger specs (https://webfinger.net/) for details.
        .route("/.well-known/webfinger", get(webfinger::finger))
        //  The profile of a user. It could be any url as it is referenced
        //  from the webfinger handler.
        .merge(activitypub::router())
        .nest("/auth", auth::routes())
        .layer(TraceLayer::new_for_http())
        .layer(Extension(db))
        .layer(Extension(templates));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

