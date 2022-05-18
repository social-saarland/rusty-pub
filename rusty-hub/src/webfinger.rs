use reqwest;
use reqwest::header::{HeaderMap, ACCEPT};
use url::Url;
use urlencoding::encode;

use serde::{Deserialize, Serialize};

use serde_json::Value;
use std::collections::HashMap;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Link {
    pub rel: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiles: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, String>>,

    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Finger {
    pub subject: Option<String>,
    pub aliases: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, String>>,
    pub links: Vec<Link>,

    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl Finger {
    pub async fn query(domain: &str, user: &str) -> Option<Finger> {
        let mut url = Url::parse(&format!("https://{}/.well-known/webfinger", domain)).unwrap();
        url.set_query(Some(&format!(
            "resource={}",
            encode(&format!("acct:{}", user))
        )));

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/jrd+json".parse().unwrap());

        let client = reqwest::Client::new();
        let resp = client.get(url).headers(headers).send().await.unwrap();

        Some(resp.json().await.unwrap())
    }
}

pub struct Jrd<T: Serialize>(pub T);

impl<T: Serialize> IntoResponse for Jrd<T> {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(
            "content-type",
            "application/jrd+json; charset=utf-8".parse().unwrap(),
        );
        (
            StatusCode::OK,
            headers,
            serde_json::to_string(&self.0).unwrap(),
        )
            .into_response()
    }
}
