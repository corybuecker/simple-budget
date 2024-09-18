use bson::{doc, oid::ObjectId, serde_helpers::hex_string_as_object_id, Bson};
use chrono::{DateTime, Utc};
use core::fmt;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug)]
pub struct NotFoundError {}

impl fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", 1)
    }
}

impl Error for NotFoundError {}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum GoalHeader {
    Accumulated,
    DaysRemaining,
}

impl From<GoalHeader> for Bson {
    fn from(goal_header: GoalHeader) -> Bson {
        match goal_header {
            GoalHeader::Accumulated => String::from("Accumulated").into(),
            GoalHeader::DaysRemaining => String::from("DaysRemaining").into(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Preferences {
    pub timezone: Option<String>,
    pub goal_header: Option<GoalHeader>,
    pub forecast_offset: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Session {
    #[serde(with = "hex_string_as_object_id")]
    pub _id: String,

    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub expiration: DateTime<Utc>,
    pub id: bson::Uuid,
    pub csrf: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    #[serde(with = "hex_string_as_object_id")]
    pub _id: String,

    pub email: String,
    pub subject: String,
    pub sessions: Option<Vec<Session>>,
    pub preferences: Preferences,
}

impl User {
    pub async fn get_by_id(client: &mongodb::Client, id: &str) -> Result<Self, Box<dyn Error>> {
        let id = ObjectId::parse_str(id)?;
        let database = client.default_database().unwrap();
        let collection: Collection<Self> = database.collection("users");
        match collection.find_one(doc! {"_id": id}).await? {
            Some(user) => Ok(user),
            None => Err(Box::new(NotFoundError {})),
        }
    }
}
