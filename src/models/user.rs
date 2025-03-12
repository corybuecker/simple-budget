use anyhow::Result;
use chrono::{DateTime, Utc};
use core::fmt;
use postgres_types::Json;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio_postgres::Client;
use uuid::Uuid;

#[derive(Debug)]
pub struct NotFoundError {}

impl fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", 1)
    }
}

impl Error for NotFoundError {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GoalHeader {
    Accumulated,
    DaysRemaining,
    PerDay,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preferences {
    pub timezone: Option<String>,
    pub goal_header: Option<GoalHeader>,
    pub forecast_offset: Option<i64>,
}

impl Preferences {
    pub fn default() -> Self {
        Self {
            timezone: Some("Utc".to_owned()),
            goal_header: Some(GoalHeader::Accumulated),
            forecast_offset: Some(1),
        }
    }
}

#[derive(Debug)]
pub struct Session {
    pub id: Option<Uuid>,
    pub user_id: i32,
    pub expiration: DateTime<Utc>,
    pub csrf: String,
}

impl TryInto<Session> for tokio_postgres::Row {
    type Error = anyhow::Error;

    fn try_into(self: tokio_postgres::Row) -> Result<Session> {
        Ok(Session {
            id: self.try_get("id")?,
            user_id: self.try_get("user_id")?,
            expiration: self.try_get("expiration")?,
            csrf: self.try_get("csrf")?,
        })
    }
}

impl Session {
    pub async fn delete_expired(client: &Client) -> Result<u64> {
        let rows = client
            .execute("DELETE FROM sessions WHERE expiration < NOW()", &[])
            .await?;

        Ok(rows)
    }

    pub async fn get_by_id(client: &Client, id: &str) -> Result<Self> {
        let id = Uuid::parse_str(id)?;
        client
            .query_one("SELECT * FROM sessions WHERE id = $1", &[&id])
            .await?
            .try_into()
    }

    pub async fn create(self: &mut Session, client: &Client) -> Result<()> {
        let id = client
            .query_one(
                "INSERT INTO sessions (id, user_id, expiration, csrf) VALUES ($1, $2, $3, $4) RETURNING id",
                &[&Uuid::new_v4(), &self.user_id, &self.expiration, &self.csrf],
            )
            .await.unwrap();

        self.id = Some(id.get::<_, Uuid>("id"));

        Ok(())
    }
}

pub struct User {
    pub id: i32,
    pub email: String,
    pub subject: String,
    pub sessions: Option<Vec<Session>>,
    pub preferences: Option<Json<Preferences>>,
}

impl TryInto<User> for tokio_postgres::Row {
    type Error = anyhow::Error;

    fn try_into(self: tokio_postgres::Row) -> Result<User> {
        Ok(User {
            id: self.try_get("id")?,
            email: self.try_get("email")?,
            subject: self.try_get("subject")?,
            sessions: None,
            preferences: self.try_get("preferences")?,
        })
    }
}

impl User {
    pub async fn create(client: &Client, email: String, subject: String) -> Result<Self> {
        let id = client
            .query_one(
                "INSERT INTO users (email, subject) VALUES ($1, $2) RETURNING id",
                &[&email, &subject],
            )
            .await?;

        let id: i32 = id.get("id");

        Self::get_by_id(client, id).await
    }

    pub async fn update(&self, client: &Client) -> Result<Self> {
        let id = client
            .query_one(
                "UPDATE users SET preferences = $1 WHERE id = $2 RETURNING id",
                &[&self.preferences, &self.id],
            )
            .await?;

        let id: i32 = id.get("id");

        Self::get_by_id(client, id).await
    }

    pub async fn get_by_subject(client: &Client, subject: String) -> Result<Self> {
        client
            .query_one("SELECT * FROM users WHERE subject = $1", &[&subject])
            .await?
            .try_into()
    }

    pub async fn get_by_id(client: &Client, id: i32) -> Result<Self> {
        client
            .query_one("SELECT * FROM users WHERE id = $1", &[&id])
            .await?
            .try_into()
    }
}
