use serde_json;

#[derive(Serialize, Deserialize)]
struct Address {
    #[serde(rename = "Email")]
    email: String,
    #[serde(rename = "Name")]
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Message {
    #[serde(rename = "From")]
    from: Address,
    #[serde(rename = "To")]
    to: Vec<Address>,
    #[serde(rename = "Subject")]
    subject: String,
    #[serde(rename = "TextPart")]
    text: String,
    #[serde(rename = "HTMLPart")]
    html: String,
    #[serde(rename = "CustomID")]
    custom_id: String,
}

#[derive(Serialize, Deserialize)]
struct Messages {
    #[serde(rename = "Messages")]
    messages: Vec<Message>,
}

pub struct MailJet {
    app_url: String,
    api_url: String,
    username: String,
    password: String,
}

impl MailJet {
    pub fn from_env() -> MailJet {
        MailJet {
            api_url: env::var("MAILJET_URL").expect("'MAILJET_URL' must be set"),
            username: env::var("MAILJET_USER").expect("'MAILJET_USER' must be set"),
            password: env::var("MAILJET_PWD").expect("'MAILJET_PWD' must be set"),
            app_url: env::var("APP_URL").expect("'APP_URL' must be set"),
        }
    }

    pub async fn send(self: &MailJet, messages: &Messages) -> Result<(), ()> {
        let client = reqwest::Client::new();

        let req = client
            .post(&self.mailjet_url)
            .basic_auth(&self.mailjet_user, Some(&self.mailjet_pwd))
            .json(&messages)
            .send()
            .await?;
    }
}
/*
    let msg = Message {
        to: vec![Address {
            name: "Achim Domma".to_string(),
            email: "achim@domma.de".to_string(),
        }],
        from: Address {
            name: "Achim Domma".to_string(),
            email: "achim@domma.de".to_string(),
        },

        subject: "My first Rust email".to_string(),
        text: "Greetings from Rust!!!.".to_string(),
        html: "<h3>Rust!!!</h3><br   />May the Rust force be with you!".to_string(),
        custom_id: "PubHub".to_string(),
    };

    let jet = MailJet {
        messages: vec![msg],
    };


    HttpResponse::Ok().body("ok")
*/
