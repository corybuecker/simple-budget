use crate::{errors::AppError, utilities::dates::Times};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Days, Months, TimeDelta, Utc};
use postgres_types::{FromSql, ToSql};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde::Serialize;
use tokio_postgres::Client;

#[derive(Debug, Clone, Serialize, FromSql, ToSql, PartialEq)]
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
#[derive(Serialize, Debug, Clone)]
pub struct Goal {
    pub id: Option<i32>,
    pub user_id: i32,
    pub name: String,
    pub recurrence: Recurrence,
    pub target_date: DateTime<Utc>,
    pub target: Decimal,
    pub accumulated_amount: Decimal,
    pub start_date: Option<DateTime<Utc>>,
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
            accumulated_amount: self
                .try_get("accumulated_amount")
                .map_err(AppError::RecordDeserializationError)?,
            start_date: self
                .try_get("start_date")
                .map_err(AppError::RecordDeserializationError)?,
        })
    }
}

impl Goal {
    pub async fn create(&self, client: &Client) -> Result<Self, AppError> {
        let row = client
            .query_one(
                "INSERT INTO goals (
                    user_id
                    , name
                    , recurrence
                    , target_date
                    , target
                    , accumulated_amount
                    , start_date
                ) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
                &[
                    &self.user_id,
                    &self.name,
                    &self.recurrence,
                    &self.target_date,
                    &self.target,
                    &Decimal::ZERO,
                    &self.start_date,
                ],
            )
            .await?;

        let mut new_account = self.clone();
        new_account.id = Some(row.try_get("id")?);

        Ok(new_account)
    }

    pub async fn update(&self, client: &Client) -> Result<Self, AppError> {
        client
            .execute(
                "UPDATE goals SET
                    name = $1
                    , recurrence = $2
                    , target_date = $3
                    , target = $4
                    , accumulated_amount = $5
                    , start_date = $6
            WHERE id = $7 AND user_id = $8",
                &[
                    &self.name,
                    &self.recurrence,
                    &self.target_date,
                    &self.target,
                    &self.accumulated_amount,
                    &self.start_date,
                    &self.id,
                    &self.user_id,
                ],
            )
            .await?;

        let goal = Goal::get_one(
            client,
            self.id.ok_or(anyhow!("missing ID after update"))?,
            self.user_id,
        )
        .await?;

        Ok(goal)
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
            JOIN users ON users.id = goals.user_id WHERE users.id = $1
            ORDER BY target_date ASC",
                &[&user_id],
            )
            .await?;

        let mut goals = Vec::with_capacity(rows.len());
        for row in rows {
            goals.push(row.try_into()?);
        }

        Ok(goals)
    }

    pub async fn get_all_unscoped(client: &Client) -> Result<Vec<Self>, AppError> {
        let rows = client
            .query(
                "SELECT goals.* FROM goals ORDER BY target_date ASC FOR UPDATE",
                &[],
            )
            .await?;

        let mut goals = Vec::with_capacity(rows.len());
        for row in rows {
            goals.push(row.try_into()?);
        }

        Ok(goals)
    }

    pub async fn get_expired(
        client: &Client,
        cutoff: DateTime<Utc>,
    ) -> Result<Vec<Self>, AppError> {
        let rows = client
            .query(
                "SELECT goals.* FROM goals WHERE recurrence <> 'Never' AND target_date < $1",
                &[&cutoff],
            )
            .await?;

        let mut goals = Vec::with_capacity(rows.len());
        for row in rows {
            goals.push(row.try_into()?);
        }

        Ok(goals)
    }

    pub fn increment(&self) -> Result<Self> {
        let mut goal = self.clone();
        goal.accumulated_amount = Decimal::ZERO;

        match self.recurrence {
            Recurrence::Never => goal.target_date = self.target_date,
            Recurrence::Daily => {
                goal.target_date = self
                    .target_date
                    .checked_add_days(Days::new(1))
                    .ok_or_else(|| anyhow!("could not add dates"))?
            }
            Recurrence::Weekly => {
                goal.target_date = self
                    .target_date
                    .checked_add_days(Days::new(7))
                    .ok_or_else(|| anyhow!("could not add dates"))?
            }
            Recurrence::Yearly => {
                goal.target_date = self
                    .target_date
                    .checked_add_months(Months::new(12))
                    .ok_or_else(|| anyhow!("could not add dates"))?
            }
            Recurrence::Monthly => {
                goal.target_date = self
                    .target_date
                    .checked_add_months(Months::new(1))
                    .ok_or_else(|| anyhow!("could not add dates"))?
            }
            Recurrence::Quarterly => {
                goal.target_date = self
                    .target_date
                    .checked_add_months(Months::new(3))
                    .ok_or_else(|| anyhow!("could not add dates"))?
            }
        }

        Ok(goal)
    }

    pub fn accumulated_per_day(&self) -> Result<Decimal> {
        if self.start_at()? > Utc::now() {
            return Ok(Decimal::ZERO);
        }

        if self.accumulated_amount >= self.target {
            return Ok(Decimal::ZERO);
        }

        let total_time_in_days = Decimal::from_i64(self.total_time()?.num_days())
            .ok_or(anyhow!("could not convert decimal"))?;

        Ok(self.target / total_time_in_days)
    }

    pub async fn accelerate(&self, client: &Client, amount: Decimal) -> Result<Self, AppError> {
        let mut goal = self.clone();
        goal.accumulated_amount += amount;

        if goal.accumulated_amount >= goal.target {
            goal.accumulated_amount = goal.target;
        }

        goal.update(client).await
    }

    pub async fn accumulate(
        &self,
        client: &Client,
        time_provider: &impl Times,
    ) -> Result<Self, AppError> {
        let accumulated_now = self.accumulated_now(time_provider)?;
        let accumulated_amount = Decimal::min(
            self.target,
            Decimal::max(accumulated_now, self.accumulated_amount),
        );

        let goal = Goal {
            id: self.id,
            target: self.target,
            target_date: self.target_date,
            recurrence: self.recurrence.clone(),
            name: self.name.clone(),
            user_id: self.user_id,
            accumulated_amount,
            start_date: self.start_date,
        };

        goal.update(client).await
    }

    fn accumulated_now(&self, time_provider: &impl Times) -> Result<Decimal> {
        if self.start_at()? > time_provider.now() {
            return Ok(Decimal::ZERO);
        }

        if time_provider.now() > self.target_date {
            return Ok(self.target);
        }

        if self.accumulated_amount >= self.target {
            return Ok(self.target);
        }

        let elapsed_time_in_seconds =
            Decimal::from_i64(self.elapsed_time(time_provider)?.num_seconds())
                .ok_or(anyhow!("could not convert decimal"))?;

        let total_time_in_seconds = Decimal::from_i64(self.total_time()?.num_seconds())
            .ok_or(anyhow!("could not convert decimal"))?;

        Ok(self.target / total_time_in_seconds * elapsed_time_in_seconds)
    }

    fn total_time(&self) -> Result<TimeDelta> {
        Ok(self.target_date - self.start_at()?)
    }

    fn elapsed_time(&self, time_provider: &impl Times) -> Result<TimeDelta> {
        let start_at = self.start_at()?;

        Ok(time_provider.now() - start_at)
    }

    fn start_at(&self) -> Result<DateTime<Utc>> {
        match self.recurrence {
            Recurrence::Never => self.start_date.ok_or_else(|| anyhow!("missing start date")),
            Recurrence::Daily => Ok(self.target_date - Days::new(1)),
            Recurrence::Weekly => Ok(self.target_date - Days::new(7)),
            Recurrence::Yearly => Ok(self.target_date - Months::new(12)),
            Recurrence::Monthly => Ok(self.target_date - Months::new(1)),
            Recurrence::Quarterly => Ok(self.target_date - Months::new(3)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{Goal, Recurrence};
    use crate::{test_utils::state_for_tests, utilities::dates::Times};
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
    use rust_decimal::Decimal;
    use tokio_postgres::GenericClient;

    struct MockTimeProvider;
    impl Times for MockTimeProvider {
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            Utc.with_ymd_and_hms(2024, 1, 30, 0, 0, 0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap()
        }
    }

    #[tokio::test]
    async fn test_accumulate_from_over_accumulated() {
        let (shared_state, user_extension, _context_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;
        let client = shared_state.pool.get().await.unwrap();
        let client = client.client();
        let time_provider = &MockTimeProvider {};
        let goal = Goal {
            id: None,
            accumulated_amount: Decimal::new(90, 0),
            name: "test".to_string(),
            recurrence: Recurrence::Monthly,
            target: Decimal::new(100, 0),
            user_id,
            target_date: NaiveDateTime::new(
                NaiveDate::from_str("2024-02-15").unwrap(),
                NaiveTime::MIN,
            )
            .and_utc(),
            start_date: None,
        };
        let goal = goal.create(client).await.unwrap();
        let goal = goal.accumulate(client, time_provider).await.unwrap();

        assert_eq!(goal.accumulated_amount, Decimal::new(90, 0))
    }

    #[tokio::test]
    async fn test_accumulate_from_over_target() {
        let (shared_state, user_extension, _context_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;
        let client = shared_state.pool.get().await.unwrap();
        let client = client.client();
        let time_provider = &MockTimeProvider {};
        let goal = Goal {
            id: None,
            accumulated_amount: Decimal::new(101, 0),
            name: "test".to_string(),
            recurrence: Recurrence::Monthly,
            target: Decimal::new(100, 0),
            user_id,
            target_date: NaiveDateTime::new(
                NaiveDate::from_str("2024-01-29").unwrap(),
                NaiveTime::MIN,
            )
            .and_utc(),
            start_date: None,
        };
        let goal = goal.create(client).await.unwrap();
        let goal = goal.accumulate(client, time_provider).await.unwrap();

        assert_eq!(goal.accumulated_amount, Decimal::new(100, 0))
    }

    #[tokio::test]
    async fn test_accumulate_from_zero() {
        let (shared_state, user_extension, _context_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;
        let client = shared_state.pool.get().await.unwrap();
        let client = client.client();
        let time_provider = &MockTimeProvider {};
        let goal = Goal {
            id: None,
            accumulated_amount: Decimal::ZERO,
            name: "test".to_string(),
            recurrence: Recurrence::Monthly,
            target: Decimal::new(100, 0),
            user_id,
            target_date: NaiveDateTime::new(
                NaiveDate::from_str("2024-01-31").unwrap(),
                NaiveTime::MIN,
            )
            .and_utc(),
            start_date: None,
        };
        let goal = goal.create(client).await.unwrap();

        let goal = goal.accumulate(client, time_provider).await.unwrap();
        assert!(goal.accumulated_amount - Decimal::new(9766, 2) < Decimal::new(3, 1))
    }
}
