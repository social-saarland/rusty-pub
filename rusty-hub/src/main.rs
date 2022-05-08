use std::path;

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
    routing::{get,post},
    Router,
    http::StatusCode,
    async_trait,
    extract::{FromRequest, RequestParts, Query, Path, Json},
};

use tower_http::trace::TraceLayer;
use tracing_subscriber;
use dotenv::dotenv;


use rsa;
use rand;
//use rsa::pkcs8::{DecodePublicKey, EncodePublicKey, DecodePrivateKey, EncodePrivateKey};
//use rsa::pkcs8::PublicKey;
use rsa::pkcs8::LineEnding;
use rsa::pkcs8::{EncodePublicKey, EncodePrivateKey, DecodePublicKey};


struct User {
    name: String
}

impl User {
    fn new(name: &str) -> User {
        User{name: name.to_string()}
    }

    fn profile(self: &User) -> Profile {

        let private_key_path = format!("./user_keys/{}.private.pem", self.name);
        let public_key_path = format!("./user_keys/{}.public.pem", self.name);

        if !path::Path::new(&private_key_path).exists() {
            let mut rng = rand::thread_rng();

            let bits = 2048;
            let private_key = rsa::RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
            let public_key = rsa::RsaPublicKey::from(&private_key);

            public_key.write_public_key_pem_file(&public_key_path, LineEnding::default()).unwrap();
            private_key.write_pkcs8_pem_file(&private_key_path, LineEnding::default()).unwrap();
        }

        let public_key = rsa::RsaPublicKey::read_public_key_pem_file(&public_key_path).unwrap();

        let user_id = format!("https://achim.eu.ngrok.io/user/{}", &self.name).to_string();

        Profile {
            context: Some(serde_json::Value::String("https://www.w3.org/ns/activitystreams".to_string())),
            id: Some(user_id.clone()),
            name: Some(format!("Fake {}",self.name).to_string()),
            type_: Some("Person".to_string()),
            preferred_username: Some(format!("{}", self.name).to_string()),
            summary: Some("summary".to_string()),
            inbox: Some(format!("https://achim.eu.ngrok.io/user/{}/inbox", self.name).to_string()),
            outbox: Some(format!("https://achim.eu.ngrok.io/user/{}/outbox", self.name).to_string()),
            public_key: Some(PublicKey {
                id: format!("{}#main-key", &user_id).to_string(),
                owner: user_id,
                public_key_pem: public_key.to_public_key_pem(LineEnding::default()).unwrap()
                }),



            ..Default::default()
        }
    }
}

#[async_trait]
impl<B> FromRequest<B> for User
where B: Send {
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {

        let Path(user) = Path::<String>::from_request(req).await.unwrap();

        Ok(User{ name: user})
    }
}

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

impl Finger {
    async fn query(domain: &str, user: &str) -> Option<Finger> {
        let mut url = Url::parse(
            &format!("https://{}/.well-known/webfinger", domain)
        ).unwrap();
        url.set_query(Some(
            &format!("resource={}", encode(&format!("acct:{}", user)))
        ));

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/jrd+json".parse().unwrap());
        
        let client = reqwest::Client::new();
        let resp = client.get(url).headers(headers).send().await.unwrap();

        Some(resp.json().await.unwrap())
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct PublicKey {
    id: String,
    owner: String,
    #[serde(rename = "publicKeyPem")]
    public_key_pem: String
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Action {
    Follow {
        #[serde(rename = "@context")]
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        actor: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        object: Option<String>
    }
}
// {"@context":"https://www.w3.org/ns/activitystreams","actor":"https://social.saarland/users/achim","id":"https://social.saarland/31c368e7-8336-47ae-ac90-18f72b34a159","object":"https://achim.eu.ngrok.io/user/achim","type":"Follow"}


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
    #[serde(skip_serializing_if = "Option::is_none", rename = "preferredUsername")]
    preferred_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "publicKey")]
    public_key: Option<PublicKey>,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

impl Profile {
    async fn from_id(id: &str) -> Profile {
        let accept = "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"";
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, accept.parse().unwrap());
        
        let client = reqwest::Client::new();
        client.get(id).headers(headers).send().await.unwrap().json().await.unwrap()
    }
}


async fn finger(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let query = params.get("resource").unwrap();
    println!("query: {}", query);
    let x = query.split(":").collect::<Vec<&str>>();
    let y = x[1].split("@").collect::<Vec<&str>>();
    let name = y[0];

    let profile = format!("https://achim.eu.ngrok.io/user/{}", name).to_string();
    println!("profile: {}", profile);

    Jrd(
        Finger{
            subject: Some(params.get("resource").unwrap().to_string()),
/*            aliases: Some(vec![
                "https://achim.eu.ngrok.io/profile".to_string()
            ]),*/
            links: vec![
                Link {
                    rel: Some("self".to_string()),
                    type_: Some("application/activity+json".to_string()),
                    href: Some(profile),
                    ..Default::default()
                }
            ],
            ..Default::default()
        }
    )
}

struct Activity<A: Serialize>(A);

impl<A: Serialize> IntoResponse for Activity<A> {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/activity+json; charset=utf-8".parse().unwrap());
        (StatusCode::OK, headers, serde_json::to_string(&self.0).unwrap()).into_response()
    }

}

struct Jrd<T: Serialize>(T);

impl<T: Serialize> IntoResponse for Jrd<T> {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/jrd+json; charset=utf-8".parse().unwrap());
        (StatusCode::OK, headers, serde_json::to_string(&self.0).unwrap()).into_response()
    }

}

async fn profile_handler(user: User) -> Activity<Profile> {
    println!("GET: profile of user '{}'.", user.name);
    Activity(user.profile())
}

async fn inbox_post_handler(user: User, Json(payload): Json<Action>) -> impl IntoResponse  {
    println!("POST: inbox of user '{}'.", user.name);
    println!("Body: {:?}", payload);
    (StatusCode::OK, "inbox").into_response()
}

async fn inbox_get_handler(user: User) -> impl IntoResponse  {
    println!("GET: inbox of user '{}'.", user.name);
    (StatusCode::OK, "inbox").into_response()
}

async fn outbox_post_handler(user: User) -> impl IntoResponse  {
    println!("POST: outbox of user '{}'.", user.name);
    (StatusCode::OK, "outbox").into_response()
}

async fn outbox_get_handler(user: User) -> impl IntoResponse  {
    println!("GET: outbox of user '{}'.", user.name);
    (StatusCode::OK, "outbox").into_response()
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
//    let f = Finger::query("social.saarland", "achim@social.saarland").await.unwrap();
//    println!("{:?}", f);
    
//    let p = Profile::from_id("https://social.saarland/users/achim").await;
//    println!("{:?}", p.type_);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/.well-known/webfinger", get(finger))
        .route("/user/:user", get(profile_handler))
        .route("/user/:user/inbox", post(inbox_post_handler))
        .route("/user/:user/inbox", get(inbox_get_handler))
        .route("/user/:user/outbox", post(outbox_post_handler))
        .route("/user/:user/outbox", get(outbox_get_handler))
        .layer(TraceLayer::new_for_http());

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
