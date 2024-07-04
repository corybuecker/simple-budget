use bson::{doc, oid::ObjectId};
use chrono::{DateTime, Datelike, Days, Local, Months, TimeDelta, TimeZone, Timelike, Utc};
use mongodb::{Client, Collection};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all(deserialize = "lowercase"))]
pub enum Recurrence {
    Never,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Goal {
    recurrence: Recurrence,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    target_date: DateTime<Utc>,
    target: f64,
}

pub async fn goals(
    client: &Client,
    user_id: &ObjectId,
) -> Result<Vec<Goal>, mongodb::error::Error> {
    let mut goals: Vec<Goal> = Vec::new();

    let collection: Collection<Goal> = client.database("simple_budget").collection("goals");
    let mut cursor = collection.find(doc! {"user_id": user_id}).await?;

    while cursor.advance().await? {
        goals.push(cursor.deserialize_current()?);
    }

    Ok(goals)
}

impl Goal {
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
        let elapsed_time = Local::now() - start_at;
        elapsed_time
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
