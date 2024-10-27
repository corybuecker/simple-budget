use crate::errors::ModelError;
use bson::{doc, oid::ObjectId, serde_helpers::hex_string_as_object_id};
use chrono::{DateTime, Datelike, Days, Duration, Local, Months, TimeDelta, Timelike, Utc};
use mongodb::results::{InsertOneResult, UpdateResult};
use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
#[serde(rename_all(deserialize = "lowercase", serialize = "lowercase"))]
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Goal {
    #[serde(with = "hex_string_as_object_id")]
    pub _id: String,

    #[serde(with = "hex_string_as_object_id")]
    pub user_id: String,

    pub name: String,
    pub recurrence: Recurrence,

    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub target_date: DateTime<Utc>,

    pub target: f64,
}

impl Goal {
    pub async fn create(&self, client: &mongodb::Client) -> Result<InsertOneResult, ModelError> {
        Ok(client
            .default_database()
            .ok_or_else(|| ModelError::MissingDefaultDatabase)?
            .collection::<Goal>("goals")
            .insert_one(self)
            .await?)
    }

    pub async fn update(&self, client: &mongodb::Client) -> Result<UpdateResult, ModelError> {
        Ok(client
            .default_database()
            .ok_or_else(|| ModelError::MissingDefaultDatabase)?
            .collection::<Goal>("goals")
            .replace_one(
                doc! {"_id": ObjectId::parse_str(&self._id)?, "user_id": ObjectId::parse_str(&self.user_id)?},
                self
            )
            .await?)
    }

    pub async fn get_by_user_id(
        client: &mongodb::Client,
        user_id: &str,
    ) -> Result<Vec<Self>, ModelError> {
        let database = client
            .default_database()
            .ok_or(ModelError::MissingDefaultDatabase)?;

        let user_id = ObjectId::parse_str(user_id).map_err(ModelError::OidParsingError)?;

        let collection = database.collection::<Goal>("goals");

        let mut cursor = collection.find(doc! {"user_id": &user_id}).await?;
        let mut goals = Vec::new();
        while cursor.advance().await? {
            let goal = cursor.deserialize_current()?;

            goals.push(goal);
        }

        Ok(goals)
    }

    pub fn increment(&self) -> Self {
        let mut goal = Goal {
            _id: self._id.clone(),
            target: self.target,
            user_id: self.user_id.clone(),
            recurrence: self.recurrence,
            name: self.name.clone(),
            target_date: self.target_date,
        };

        match self.recurrence {
            Recurrence::Never => goal.target_date = self.target_date,
            Recurrence::Daily => goal.target_date = self.target_date.add(Duration::days(1)),
            Recurrence::Weekly => goal.target_date = self.target_date.add(Duration::weeks(1)),
            Recurrence::Yearly => goal.target_date = self.target_date.add(Duration::days(365)),
            Recurrence::Monthly => goal.target_date = self.target_date.add(Duration::days(30)),
            Recurrence::Quarterly => goal.target_date = self.target_date.add(Duration::weeks(12)),
        }

        goal
    }

    pub fn accumulated_per_day(&self) -> f64 {
        if self.start_at() > Local::now() {
            return 0.0;
        }

        if Local::now() > self.target_date {
            return 0.0;
        }

        self.target / self.total_time().num_days() as f64
    }

    pub fn accumulated(&self) -> f64 {
        if self.start_at() > Local::now() {
            return 0.0;
        }

        if Local::now() > self.target_date {
            return self.target;
        }

        self.target / self.total_time().num_seconds() as f64
            * self.elapsed_time().num_seconds() as f64
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
