use tokio;
use reqwest;
use reqwest::header::{HeaderMap, ACCEPT};
use url::Url;
use urlencoding::encode;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use serde_json::Value;

use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Router,
    body::Full,
    http::StatusCode,
    extract::Query
};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Link {
    rel: Option<String>,
    #[serde(rename = "type")]
    type_: Option<String>,
    href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tiles: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<HashMap<String, String>>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Finger {
    subject: Option<String>,
    aliases: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<HashMap<String, String>>,
    links: Vec<Link>,
    
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}


async fn webfinger() {
    let mut url = Url::parse("https://social.saarland/.well-known/webfinger").unwrap();
    //let params = format!("resource={}", encode("acct:@achim"));
    let params = format!("resource={}", encode("acct:achim@social.saarland"));
    url.set_query(Some(&params));

    println!("{}", url);

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/jrd+json".parse().unwrap());
    
    let client = reqwest::Client::new();

    let resp = client.get(url).headers(headers).send().await.unwrap();
    println!("{:?}", resp.headers());

    let result : Finger = resp.json().await.unwrap();

    println!("{:?}", result);
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Profile {
    #[serde(rename = "@context")]
    context: Option<Value>,
    id: Option<String>,
    #[serde(rename = "type")]
    type_: Option<String>,
    inbox: Option<String>,
    outbox: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    followers: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    following: Option<String>,
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "preferedUsername")]
    preferred_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,


    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

async fn profile() {
    let url = "https://social.saarland/users/achim";

    let accept = "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"";
    
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, accept.parse().unwrap());
    
    let client = reqwest::Client::new();
    let result : Profile = client.get(url).headers(headers).send().await.unwrap().json().await.unwrap();

    println!("{:?}", result.type_);
}

async fn finger(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    println!("ok");
    println!("{:?}", params);
    let result = Finger{
        subject: Some(params.get("resource").unwrap().to_string()),
        aliases: Some(vec![
            "https://achim.eu.ngrok.io/profile".to_string()
        ]),
        links: vec![
            Link {
                rel: Some("self".to_string()),
                type_: Some("application/activity+json".to_string()),
                href: Some("https://achim.eu.ngrok.io/profile".to_string()),
                ..Default::default()
            }
        ],
        ..Default::default()
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/jrd+json; charset=utf-8")
        .body(Full::from(serde_json::to_string(&result).unwrap()))
        .unwrap()
}

async fn fake_profile() -> impl IntoResponse {
    let profile = Profile {
        context: Some(serde_json::Value::String("https://www.w3.org/ns/activitystreams".to_string())),
        id: Some("https://achim.eu.ngrok.io/profile".to_string()),
        name: Some("Fake Achim".to_string()),
        type_: Some("Person".to_string()),
        preferred_username: Some("achim".to_string()),
        summary: Some("summary".to_string()),
        inbox: Some("https://achim.eu.ngrok.io/profile/inbox".to_string()),
        outbox: Some("https://achim.eu.ngrok.io/profile/outbox".to_string()),
        ..Default::default()
    };
    println!("profile called!");
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/activity+json; charset=utf-8")
        .body(Full::from(serde_json::to_string(&profile).unwrap()))
        .unwrap()
}

#[tokio::main]
async fn main() {
//    webfinger().await;
    profile().await;

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/.well-known/webfinger", get(finger))
        .route("/profile", get(fake_profile));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
