use crate::activitypub::PublicKey;
use axum::async_trait;
use axum::extract::{FromRequest, Path, RequestParts};
use reqwest::StatusCode;
use std::path;

use crate::activitypub::Profile;
use rand;
use rsa;
use rsa::pkcs8::LineEnding;
use rsa::pkcs8::{DecodePublicKey, EncodePrivateKey, EncodePublicKey};

//  Fake implementation of a user. This demo knows all user names and
//  the implementation is just an object holding the name.
pub struct User {
    pub name: String,
}

impl User {
    pub fn new(name: &str) -> User {
        User {
            name: name.to_string(),
        }
    }

    pub fn profile(self: &User) -> Profile {
        let private_key_path = format!("./user_keys/{}.private.pem", self.name);
        let public_key_path = format!("./user_keys/{}.public.pem", self.name);

        if !path::Path::new(&private_key_path).exists() {
            let mut rng = rand::thread_rng();

            let bits = 2048;
            let private_key =
                rsa::RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
            let public_key = rsa::RsaPublicKey::from(&private_key);

            public_key
                .write_public_key_pem_file(&public_key_path, LineEnding::default())
                .unwrap();
            private_key
                .write_pkcs8_pem_file(&private_key_path, LineEnding::default())
                .unwrap();
        }

        let public_key = rsa::RsaPublicKey::read_public_key_pem_file(&public_key_path).unwrap();

        let user_id = format!("https://achim.eu.ngrok.io/user/{}", &self.name).to_string();

        Profile {
            context: Some(serde_json::Value::String(
                "https://www.w3.org/ns/activitystreams".to_string(),
            )),
            id: Some(user_id.clone()),
            name: Some(format!("Fake {}", self.name).to_string()),
            type_: Some("Person".to_string()),
            preferred_username: Some(format!("{}", self.name).to_string()),
            summary: Some("summary".to_string()),
            inbox: Some(format!("https://achim.eu.ngrok.io/user/{}/inbox", self.name).to_string()),
            outbox: Some(
                format!("https://achim.eu.ngrok.io/user/{}/outbox", self.name).to_string(),
            ),
            public_key: Some(PublicKey {
                id: format!("{}#main-key", &user_id).to_string(),
                owner: user_id,
                public_key_pem: public_key.to_public_key_pem(LineEnding::default()).unwrap(),
            }),

            ..Default::default()
        }
    }
}

#[async_trait]
impl<B> FromRequest<B> for User
where
    B: Send,
{
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Path(user) = Path::<String>::from_request(req).await.unwrap();

        Ok(User { name: user })
    }
}
