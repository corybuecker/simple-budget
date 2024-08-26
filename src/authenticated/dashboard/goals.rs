use crate::models::goal::Goal;
use bson::{doc, oid::ObjectId};
use mongodb::{Client, Collection};

pub async fn goals(
    client: &Client,
    user_id: &ObjectId,
) -> Result<Vec<Goal>, mongodb::error::Error> {
    let mut goals: Vec<Goal> = Vec::new();

    let collection: Collection<Goal> = client.default_database().unwrap().collection("goals");
    let mut cursor = collection.find(doc! {"user_id": user_id}).await?;

    while cursor.advance().await? {
        goals.push(cursor.deserialize_current()?);
    }

    Ok(goals)
}
