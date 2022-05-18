use tokio;

use std::collections::HashMap;

use axum::{
    extract::{Json, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use dotenv::dotenv;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

pub mod activitypub;
pub mod user;
pub mod webfinger;

use activitypub::{Action, Activity, Profile};
use user::User;
use webfinger::{Finger, Jrd, Link};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    /*
     * Minimal axum based webserver, providing a Webfinger
     * endpoint and minimal ActivityPub support: User profiles,
     * inboxes and outboxes.
     */

    let app = Router::new()
        //  just some fake root to check if the app is running
        .route("/", get(|| async { "Hello, World!" }))
        //  Tthis handler is called by Mastodon if you search for a user.
        //  See the Webfinger specs (https://webfinger.net/) for details.
        .route("/.well-known/webfinger", get(finger))
        //  The profile of a user. It could be any url as it is referenced
        //  from the webfinger handler.
        .route("/user/:user", get(profile_handler))
        //  Inbox and outbox as expected by ActivityPub. Currently only the
        //  POST inbox is a bit more than just a placeholder.
        .route("/user/:user/inbox", post(inbox_post_handler))
        .route("/user/:user/inbox", get(inbox_get_handler))
        .route("/user/:user/outbox", post(outbox_post_handler))
        .route("/user/:user/outbox", get(outbox_get_handler))
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
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

//  Returns the ActivityPub profile of a user.
async fn profile_handler(user: User) -> Activity<Profile> {
    println!("GET: profile of user '{}'.", user.name);
    Activity(user.profile())
}

//  Handles messages posted to a users inbox. Currently only "Follow"
//  actions can be deserialized. They are dumped to the console for
//  demo purposes. Nothing else is happening yet.
async fn inbox_post_handler(user: User, Json(payload): Json<Action>) -> impl IntoResponse {
    println!("POST: inbox of user '{}'.", user.name);
    println!("Body: {:?}", payload);
    (StatusCode::OK, "inbox").into_response()
}

//  Other handlers are just placeholders.
//
async fn inbox_get_handler(user: User) -> impl IntoResponse {
    println!("GET: inbox of user '{}'.", user.name);
    (StatusCode::OK, "inbox").into_response()
}

async fn outbox_post_handler(user: User) -> impl IntoResponse {
    println!("POST: outbox of user '{}'.", user.name);
    (StatusCode::OK, "outbox").into_response()
}

async fn outbox_get_handler(user: User) -> impl IntoResponse {
    println!("GET: outbox of user '{}'.", user.name);
    (StatusCode::OK, "outbox").into_response()
}
