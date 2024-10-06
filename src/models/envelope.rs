use bson::doc;
use bson::oid::ObjectId;
use bson::serde_helpers::hex_string_as_object_id;
use mongodb::Collection;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug)]
pub struct Envelope {
    #[serde(with = "hex_string_as_object_id")]
    pub _id: String,

    #[serde(with = "hex_string_as_object_id")]
    pub user_id: String,

    pub name: String,
    pub amount: f64,
}

pub async fn envelopes_total_for(user_id: &ObjectId, client: &mongodb::Client) -> f64 {
    let collection: Collection<Envelope> =
        client.default_database().unwrap().collection("envelopes");

    let mut envelopes: Vec<Envelope> = Vec::new();
    match collection.find(doc! {"user_id": user_id}).await {
        Ok(mut cursor) => {
            while cursor.advance().await.unwrap() {
                match cursor.deserialize_current() {
                    Ok(envelope) => {
                        envelopes.push(envelope);
                    }
                    Err(e) => {
                        log::error!("{}", e);
                    }
                }
            }
        }
        Err(e) => {
            log::error!("{}", e);
        }
    }

    let total = envelopes
        .iter()
        .map(|e| e.amount)
        .reduce(|memo, amount| memo + amount);

    total.unwrap_or(0.0)
}
