/*use actix_web::*;

use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};

use dotenv::dotenv;
use futures_util::future::{err, ok, Ready};

use serde::{Deserialize, Serialize};

extern crate serde_json;

use uuid::Uuid;

use sqlx;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

mod db;
mod html;
mod templating;

struct OnlyLoggedIn {
    data: String,
}

impl FromRequest for OnlyLoggedIn {
    type Error = Error;
    type Future = Ready<Result<OnlyLoggedIn, Error>>;

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        println!("only-logged-in");

        if let Ok(id) = Identity::from_request(req, payload).into_inner() {
            if let Some(user) = id.identity() {
                ok(OnlyLoggedIn {
                    data: user.to_owned(),
                })
            } else {
                err(error::ErrorBadRequest("Access denied."))
            }
        } else {
            err(error::ErrorBadRequest("Access denied."))
        }
    }
}

#[get("/test")]
async fn test(pool: web::Data<PgPool>) -> impl Responder {
    let data = db::auth::get_login_attempts(&pool).await;
    html::render("test.html").data("test", data) //.data("mails", t)
}

#[get("/")]
async fn hello() -> impl Responder {
    html::render("index.html")
}

#[get("/login")]
async fn login_form(id: Identity) -> impl Responder {
    id.remember("User123".to_owned());
    html::render("login.html")
}

#[get("/login/{id}")]
async fn exec_login(
    id: web::Path<Uuid>,
    _identity: Identity,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let account = db::auth::execute_login(&pool, *id).await;

    match account {
        Ok(_a) => {
            println!("Login ok!");
            //            let u = User{id: a.id};
            //            identity.remember(serde_json::to_string(&u).unwrap());
        }
        Err(_) => {
            println!("Login failed!");
        }
    }

    HttpResponse::Found()
        .append_header(("Location", "/me"))
        .finish()
}


#[post("/login")]
async fn send_login_link(
    form: web::Form<LoginFormData>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    println!("Email: {}", form.email);

    let id = db::auth::prepare_login(pool.get_ref(), &form.email).await;

    let login_url = format!("http://localhost:8088/auth/login/{}", id);

    println!("{}", login_url);
    /*    let attempt = sqlx::query!("select prepare_login($1) as uuid", form.email)
        .fetch_one(pool.get_ref()).await.unwrap().uuid.unwrap();


    smtp.get_ref().send(&form.email, &login_url).await;*/

    html::render("link_was_sent.html").data("login_url", login_url)
}

#[get("/logout")]
async fn logout(id: Identity) -> impl Responder {
    id.forget();
    HttpResponse::Ok().body("logged out")
}

async fn protected(auth: OnlyLoggedIn) -> impl Responder {
    HttpResponse::Ok().body(auth.data)
}

async fn not_protected() -> impl Responder {
    HttpResponse::Ok().body("not logged in")
}


/*

  curl -s \
  -X POST \
  --user "4ff550e62e736b0236a4acbb380adf94:9e0f8040cd5e3c97f584b9d4374300a3" \
  https://api.mailjet.com/v3.1/send \
  -H 'Content-Type: application/json' \
  -d '{
    "Messages":[
      {
        "From": {
          "Email": "achim@domma.de",
          "Name": "Achim"
        },
        "To": [
          {
            "Email": "achim@domma.de",
            "Name": "Achim"
          }
        ],
        "Subject": "My first Mailjet email",
        "TextPart": "Greetings from Mailjet.",
        "HTMLPart": "<h3>Dear passenger 1, welcome to <a href='https://www.mailjet.com/'>Mailjet</a>!</h3><br   />May the delivery force be with you!",
        "CustomID": "AppGettingStartedTest"
      }
    ]
  }'





*/
#[get("/send")]
async fn echo() -> impl Responder {
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

*/

use axum::{
    routing::{get, post},
    Router,
    response::IntoResponse,
    extract::Form,
};

use serde::{Deserialize, Serialize};
use crate::templating::Templates;

use crate::db::Db;

#[derive(Deserialize, Serialize)]
struct LoginFormData {
    email: String,
}

async fn show_login_form(tmpl: Templates) -> impl IntoResponse {
    tmpl.render("login.html")
}

/*
async fn exec_login(
    id: web::Path<Uuid>,
    _identity: Identity,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let account = db::auth::execute_login(&pool, *id).await;

    match account {
        Ok(_a) => {
            println!("Login ok!");
            //            let u = User{id: a.id};
            //            identity.remember(serde_json::to_string(&u).unwrap());
        }
        Err(_) => {
            println!("Login failed!");
        }
    }

    HttpResponse::Found()
        .append_header(("Location", "/me"))
        .finish()
}*/


async fn send_login_link(form: Form<LoginFormData>, db: Db, t: Templates) -> impl IntoResponse {
    println!("Email: {}", form.email);

    let id = db.prepare_login(&form.email).await;

    let login_url = format!("http://localhost:3000/auth/login/{}", id);

    println!("{}", login_url);
    /*    let attempt = sqlx::query!("select prepare_login($1) as uuid", form.email)
        .fetch_one(pool.get_ref()).await.unwrap().uuid.unwrap();


    smtp.get_ref().send(&form.email, &login_url).await;*/

    t.render("link_was_sent.html").set("login_url", login_url)
}
/*
async fn logout(id: Identity) -> impl Responder {
    id.forget();
    HttpResponse::Ok().body("logged out")
}
*/
pub fn routes() -> Router {
    Router::new()
    .route("/login", get(show_login_form))
    .route("/login", post(send_login_link))
    /*.route("/login/:login_id", get(exec_login))
    .route("/logout", get(logout)*/
}
