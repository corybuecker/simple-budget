use bson::serde_helpers::hex_string_as_object_id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
}
