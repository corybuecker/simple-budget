use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use core::fmt;
use postgres_types::Json;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio_postgres::Client;
use uuid::Uuid;

use crate::errors::AppError;

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
    pub monthly_income: Option<Decimal>,
}

impl Preferences {
    pub fn default() -> Self {
        Self {
            timezone: Some("UTC".to_owned()),
            goal_header: Some(GoalHeader::Accumulated),
            forecast_offset: Some(1),
            monthly_income: Some(Decimal::ZERO),
        }
    }

    pub fn timezone(&self) -> Result<String> {
        self.timezone
            .clone()
            .or(Some("UTC".to_owned()))
            .ok_or(anyhow!("failure fetching timezone"))
    }

    pub fn monthly_income(&self) -> Result<Decimal> {
        self.monthly_income
            .or(Some(Decimal::ZERO))
            .ok_or(anyhow!("failure fetching monthly income"))
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

#[derive(Clone, Debug)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub subject: String,
    pub preferences: Option<Json<Preferences>>,
}

impl TryInto<User> for tokio_postgres::Row {
    type Error = AppError;

    fn try_into(self: tokio_postgres::Row) -> Result<User, AppError> {
        Ok(User {
            id: self
                .try_get("id")
                .map_err(AppError::RecordDeserializationError)?,
            email: self
                .try_get("email")
                .map_err(AppError::RecordDeserializationError)?,
            subject: self
                .try_get("subject")
                .map_err(AppError::RecordDeserializationError)?,
            preferences: self
                .try_get("preferences")
                .map_err(AppError::RecordDeserializationError)?,
        })
    }
}

impl User {
    pub async fn total_balance(&self, client: &Client) -> Result<Decimal> {
        let query = r#"WITH
            debt AS (
                SELECT
                    coalesce(sum(amount), 0) AS total
            FROM
                accounts
                WHERE
                    debt = TRUE AND user_id = $1
            ),
            balance AS (
                SELECT
                    coalesce(sum(amount), 0) AS total
            FROM
                accounts
                WHERE
                    debt = FALSE AND user_id = $1
            ),
            envelopes_total AS (
                SELECT
                    coalesce(sum(amount), 0) AS total
            FROM
                envelopes
                WHERE
                    user_id = $1
            ),
            accumulated_goals AS (
                SELECT
                    coalesce(sum(accumulated_amount), 0) AS total
            FROM
                goals
                WHERE
                    user_id = $1
            )
            SELECT
                (balance.total - debt.total - accumulated_goals.total - envelopes_total.total) AS total
            FROM
                balance,
                debt,
                accumulated_goals,
                envelopes_total;
        "#;

        let total_balance = client.query_one(query, &[&self.id]).await?;
        let total_balance: Decimal = total_balance.try_get("total")?;

        Ok(Decimal::max(Decimal::ZERO, total_balance))
    }

    pub async fn create(client: &Client, email: String, subject: String) -> Result<Self, AppError> {
        let id = client
            .query_one(
                "INSERT INTO users (email, subject) VALUES ($1, $2) RETURNING id",
                &[&email, &subject],
            )
            .await?;

        let id: i32 = id.get("id");

        Self::get_by_id(client, id).await
    }

    pub async fn update(&self, client: &Client) -> Result<Self, AppError> {
        let id = client
            .query_one(
                "UPDATE users SET preferences = $1 WHERE id = $2 RETURNING id",
                &[&self.preferences, &self.id],
            )
            .await?;

        let id: i32 = id.get("id");

        Self::get_by_id(client, id).await
    }

    pub async fn get_by_subject(client: &Client, subject: String) -> Result<Self, AppError> {
        client
            .query_one("SELECT * FROM users WHERE subject = $1", &[&subject])
            .await?
            .try_into()
    }

    pub async fn get_by_id(client: &Client, id: i32) -> Result<Self, AppError> {
        client
            .query_one("SELECT * FROM users WHERE id = $1", &[&id])
            .await?
            .try_into()
    }

    pub fn timezone(&self) -> Result<String> {
        match &self.preferences {
            Some(Json(preferences)) => preferences.timezone(),
            None => Ok("UTC".to_owned()),
        }
    }

    pub fn monthly_income(&self) -> Result<Decimal> {
        match &self.preferences {
            Some(Json(preferences)) => preferences.monthly_income(),
            None => Ok(Decimal::ZERO),
        }
    }
}
