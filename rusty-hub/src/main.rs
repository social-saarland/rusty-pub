use tokio;

use std::{collections::HashMap, error::Error};

use axum::{
    extract::{Extension, Query},
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
use webfinger::{Finger, Jrd, Link};

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

    let app = Router::new()
        //  just some fake root to check if the app is running
        .route("/", get(index))
        //  Tthis handler is called by Mastodon if you search for a user.
        //  See the Webfinger specs (https://webfinger.net/) for details.
        .route("/.well-known/webfinger", get(finger))
        //  The profile of a user. It could be any url as it is referenced
        //  from the webfinger handler.
        .merge(activitypub::router())
        .nest("/auth", auth::routes())
        .layer(TraceLayer::new_for_http())
        .layer(Extension(db::prepare_from_env().await?))
        .layer(Extension(templates));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn finger(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    //  Quick hack to extract the name from the query string.
    let query = params.get("resource").unwrap();
    println!("query: {}", query);
    let x = query.split(":").collect::<Vec<&str>>();
    let y = x[1].split("@").collect::<Vec<&str>>();
    let name = y[0];

    //  Calculate the url of a users profile. Currently all users exist. Should
    //  use axum functionality to build the url and the domain must be read from
    //  an environment variable of course.
    let profile = format!("https://achim.eu.ngrok.io/user/{}", name).to_string();
    println!("profile: {}", profile);

    //  This is just a playground, so we know all users and return the links
    //  to their profile as expected by Webfinger.
    Jrd(Finger {
        subject: Some(params.get("resource").unwrap().to_string()),
        links: vec![Link {
            rel: Some("self".to_string()),
            type_: Some("application/activity+json".to_string()),
            href: Some(profile),
            ..Default::default()
        }],
        ..Default::default()
    })
}
