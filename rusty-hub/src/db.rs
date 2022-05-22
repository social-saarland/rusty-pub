use sqlx;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::types::uuid::Uuid;
use serde::Serialize;

use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
};


#[derive(Serialize)]
pub struct Account {
    pub id: Uuid,
    pub primary_email_id: u32,
    pub created: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct LoginAttempt {
    pub id: i32,
    pub email_id: i32,
    pub created: DateTime<Utc>,
}

#[derive(Clone)]
pub struct Db(PgPool);

impl Db {
    pub async fn from_env() -> Result<Db, sqlx::Error> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;
        sqlx::migrate!().run(&pool).await.unwrap();
        Ok(Db(pool))
    }

    pub async fn prepare_login(self: &Db, email: &str) -> Uuid {
        sqlx::query!("select prepare_login($1)", &email)
            .fetch_one(&self.0)
            .await
            .unwrap()
            .prepare_login
            .unwrap()
    }

    pub async fn execute_login(self: &Db, uuid: Uuid) -> Result<Account, sqlx::Error> {
        sqlx::query_as_unchecked!(Account, "select * from execute_login($1)", &uuid)
            .fetch_one(&self.0)
            .await
    }


    pub async fn get_login_attempts(self: &Db) -> Vec<LoginAttempt> {
        sqlx::query_as_unchecked!(LoginAttempt, "select * from login_attempts")
            .fetch_all(&self.0)
            .await
            .unwrap()
    }
}


#[async_trait]
impl<B> FromRequest<B> for Db
where
    B: Send,
{
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match Extension::<Db>::from_request(req).await {
            Ok(Extension(db)) => Ok(db),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

