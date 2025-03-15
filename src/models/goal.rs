use crate::errors::AppError;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Datelike, Days, Local, Months, TimeDelta, Timelike, Utc};
use postgres_types::{FromSql, ToSql};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde::Serialize;
use tokio_postgres::Client;

#[derive(Debug, Clone, Serialize, FromSql, ToSql)]
pub enum Recurrence {
    Never,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}
#[derive(Debug)]
pub struct RecurrenceError {}

impl std::str::FromStr for Recurrence {
    fn from_str(string: &str) -> Result<Self, RecurrenceError> {
        match string {
            "never" => Ok(Self::Never),
            "daily" => Ok(Self::Daily),
            "weekly" => Ok(Self::Weekly),
            "monthly" => Ok(Self::Monthly),
            "quarterly" => Ok(Self::Quarterly),
            "yearly" => Ok(Self::Yearly),
            _ => Err(RecurrenceError {}),
        }
    }

    type Err = RecurrenceError;
}
#[derive(Serialize, Debug)]
pub struct Goal {
    pub id: Option<i32>,
    pub user_id: Option<i32>,
    pub name: String,
    pub recurrence: Recurrence,
    pub target_date: DateTime<Utc>,
    pub target: Decimal,
}

impl TryInto<Goal> for tokio_postgres::Row {
    type Error = AppError;

    fn try_into(self: tokio_postgres::Row) -> Result<Goal, AppError> {
        Ok(Goal {
            id: self
                .try_get("id")
                .map_err(AppError::RecordDeserializationError)?,
            user_id: self
                .try_get("user_id")
                .map_err(AppError::RecordDeserializationError)?,
            name: self
                .try_get("name")
                .map_err(AppError::RecordDeserializationError)?,
            recurrence: self
                .try_get("recurrence")
                .map_err(AppError::RecordDeserializationError)?,
            target_date: self
                .try_get("target_date")
                .map_err(AppError::RecordDeserializationError)?,
            target: self
                .try_get("target")
                .map_err(AppError::RecordDeserializationError)?,
        })
    }
}

impl Goal {
    pub async fn create(&mut self, client: &Client) -> Result<(), AppError> {
        let row = client
            .query_one(
                "INSERT INTO goals
            (user_id, name, recurrence, target_date, target)
            VALUES ($1, $2, $3, $4, $5) RETURNING id",
                &[
                    &self.user_id,
                    &self.name,
                    &self.recurrence,
                    &self.target_date,
                    &self.target,
                ],
            )
            .await?;

        self.id = Some(row.try_get("id")?);
        Ok(())
    }

    pub async fn update(&self, client: &Client) -> Result<(), AppError> {
        client.query("UPDATE goals SET name = $1, recurrence = $2, target_date = $3, target = $4 WHERE id = $5 AND user_id = $6", &[&self.name, &self.recurrence, &self.target_date, &self.target, &self.id, &self.user_id]).await?;
        Ok(())
    }

    pub async fn delete(&self, client: &Client) -> Result<(), AppError> {
        client
            .execute(
                "DELETE FROM goals WHERE user_id = $1 and id = $2",
                &[&self.user_id, &self.id],
            )
            .await?;
        Ok(())
    }

    pub async fn get_one(client: &Client, id: i32, user_id: i32) -> Result<Self, AppError> {
        let row = client
            .query_one(
                "SELECT goals.* FROM goals
                INNER JOIN users ON users.id = goals.user_id
                WHERE users.id = $1 AND goals.id = $2",
                &[&user_id, &id],
            )
            .await
            .map_err(AppError::RecordNotFound)?;

        row.try_into()
    }

    pub async fn get_all(client: &Client, user_id: i32) -> Result<Vec<Self>, AppError> {
        let rows = client
            .query(
                "SELECT goals.* FROM goals INNER
            JOIN users ON users.id = goals.user_id WHERE users.id = $1",
                &[&user_id],
            )
            .await?;

        let mut goals = Vec::with_capacity(rows.len());
        for row in rows {
            goals.push(row.try_into()?);
        }

        Ok(goals)
    }

    pub async fn get_expired(client: &Client) -> Result<Vec<Self>, AppError> {
        let rows = client
            .query(
                "SELECT goals.* FROM goals WHERE recurrence <> 'Never' AND target_date < NOW()",
                &[],
            )
            .await?;

        let mut goals = Vec::with_capacity(rows.len());
        for row in rows {
            goals.push(row.try_into()?);
        }

        Ok(goals)
    }

    pub fn increment(&self) -> Self {
        let mut goal = Goal {
            id: self.id,
            target: self.target,
            user_id: self.user_id,
            recurrence: self.recurrence.clone(),
            name: self.name.clone(),
            target_date: self.target_date,
        };

        match self.recurrence {
            Recurrence::Never => goal.target_date = self.target_date,
            Recurrence::Daily => {
                goal.target_date = self.target_date.checked_add_days(Days::new(1)).unwrap()
            }
            Recurrence::Weekly => {
                goal.target_date = self.target_date.checked_add_days(Days::new(7)).unwrap()
            }
            Recurrence::Yearly => {
                goal.target_date = self
                    .target_date
                    .checked_add_months(Months::new(12))
                    .unwrap()
            }
            Recurrence::Monthly => {
                goal.target_date = self.target_date.checked_add_months(Months::new(1)).unwrap()
            }
            Recurrence::Quarterly => {
                goal.target_date = self.target_date.checked_add_months(Months::new(3)).unwrap()
            }
        }

        goal
    }

    pub fn accumulated_per_day(&self) -> Result<Decimal> {
        if self.start_at() > Local::now() {
            return Ok(Decimal::ZERO);
        }

        if Local::now() > self.target_date {
            return Ok(Decimal::ZERO);
        }

        let total_time_in_days = Decimal::from_i64(self.total_time().num_days())
            .ok_or(anyhow!("could not convert decimal"))?;

        Ok(self.target / total_time_in_days)
    }

    pub fn accumulated(&self) -> Result<Decimal> {
        if self.start_at() > Local::now() {
            return Ok(Decimal::ZERO);
        }

        if Local::now() > self.target_date {
            return Ok(self.target);
        }

        let elapsed_time_in_seconds = Decimal::from_i64(self.elapsed_time().num_seconds())
            .ok_or(anyhow!("could not convert decimal"))?;

        let total_time_in_seconds = Decimal::from_i64(self.total_time().num_seconds())
            .ok_or(anyhow!("could not convert decimal"))?;

        Ok(self.target / total_time_in_seconds * elapsed_time_in_seconds)
    }

    fn total_time(&self) -> TimeDelta {
        DateTime::from(self.target_date) - self.start_at()
    }

    fn elapsed_time(&self) -> TimeDelta {
        let start_at = self.start_at();

        Local::now() - start_at
    }

    fn start_at(&self) -> DateTime<Local> {
        match self.recurrence {
            Recurrence::Never => Self::start_of_month().unwrap(),
            Recurrence::Daily => DateTime::from(self.target_date) - Days::new(1),
            Recurrence::Weekly => DateTime::from(self.target_date) - Days::new(7),
            Recurrence::Yearly => DateTime::from(self.target_date) - Months::new(12),
            Recurrence::Monthly => DateTime::from(self.target_date) - Months::new(1),
            Recurrence::Quarterly => DateTime::from(self.target_date) - Months::new(3),
        }
    }

    fn start_of_month() -> Result<DateTime<Local>, String> {
        let now = Local::now();
        let now = now.with_hour(0).ok_or("could not set time");
        let now = now?.with_minute(0).ok_or("could not set time");
        let now = now?.with_second(0).ok_or("could not set time");
        let now = now?.with_day0(0).ok_or("could not set time");
        Ok(now?)
    }
}
