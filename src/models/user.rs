use anyhow::Result;
use chrono::{DateTime, Utc};
use core::fmt;
use postgres_types::Json;
use serde::Deserialize;
use std::error::Error;
use tokio_postgres::{Client, row};

#[derive(Debug)]
pub struct NotFoundError {}

impl fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", 1)
    }
}

impl Error for NotFoundError {}

#[derive(Debug, Deserialize)]
pub enum GoalHeader {
    Accumulated,
    DaysRemaining,
    PerDay,
}

#[derive(Deserialize)]
pub struct Preferences {
    pub timezone: Option<String>,
    pub goal_header: Option<GoalHeader>,
    pub forecast_offset: Option<i64>,
}

pub struct Session {
    pub id: Option<i32>,
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

    pub async fn get_by_id(client: &Client, session_id: i32) -> Result<Self> {
        client
            .query_one("SELECT * FROM sessions WHERE id = $1", &[&session_id])
            .await?
            .try_into()
    }

    pub async fn create(self: &mut Session, client: &Client) -> Result<()> {
        let id = client
            .execute(
                "INSERT INTO sessions (user_id, expiration, csrf) VALUES ($1, $2, $3) RETURNING id",
                &[&self.user_id, &self.expiration, &self.csrf],
            )
            .await?;

        self.id = Some(id as i32);
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
