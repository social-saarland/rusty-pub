use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use reqwest;
use reqwest::header::{HeaderMap, ACCEPT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

//  This file contains data structures to serialize and deserialize
//  ActivityPub data into / from Json using the serde library.


#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Action {
    //  Manual implementation of "Follow" for demo purposes. The
    //  idea is to generate all required actions based on
    //  https://www.w3.org/TR/activitystreams-vocabulary/
    Follow {
        #[serde(rename = "@context")]
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        actor: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        object: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Profile {
    #[serde(rename = "@context")]
    pub context: Option<Value>,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub inbox: Option<String>,
    pub outbox: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followers: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub following: Option<String>,
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "preferredUsername")]
    pub preferred_username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "publicKey")]
    pub public_key: Option<PublicKey>,

    //  This will handle all unknown keys.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl Profile {
    pub async fn from_id(id: &str) -> Profile {
        //  Retrieves a user profile (from a different server) via http. User
        //  id is the url of the profile, as returned via webfinger for example.
        //  This code could be use to get the profile (i.e. inbox and outbox)
        //  of a Mastodon user.
        let accept = "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"";
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, accept.parse().unwrap());

        let client = reqwest::Client::new();
        client
            .get(id)
            .headers(headers)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}

//  Convenient wrapper to return ActivityPub data structures
//  from an axum handler.
pub struct Activity<A: Serialize>(pub A);

impl<A: Serialize> IntoResponse for Activity<A> {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(
            "content-type",
            "application/activity+json; charset=utf-8".parse().unwrap(),
        );
        (
            StatusCode::OK,
            headers,
            serde_json::to_string(&self.0).unwrap(),
        )
            .into_response()
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PublicKey {
    pub id: String,
    pub owner: String,
    #[serde(rename = "publicKeyPem")]
    pub public_key_pem: String,
}
