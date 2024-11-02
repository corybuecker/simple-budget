use crate::errors::ModelError;
use bson::doc;
use bson::oid::ObjectId;
use bson::serde_helpers::hex_string_as_object_id;
use mongodb::results::InsertOneResult;
use mongodb::results::UpdateResult;
use mongodb::Collection;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug)]
pub struct Account {
    #[serde(with = "hex_string_as_object_id")]
    pub _id: String,

    #[serde(with = "hex_string_as_object_id")]
    pub user_id: String,

    pub name: String,
    pub amount: f64,
    pub debt: bool,
}

impl Account {
    pub async fn create(&self, client: &mongodb::Client) -> Result<InsertOneResult, ModelError> {
        Ok(client
            .default_database()
            .ok_or_else(|| ModelError::MissingDefaultDatabase)?
            .collection::<Account>("accounts")
            .insert_one(self)
            .await?)
    }
    pub async fn update(&self, client: &mongodb::Client) -> Result<UpdateResult, ModelError> {
        Ok(client
            .default_database()
            .ok_or_else(|| ModelError::MissingDefaultDatabase)?
            .collection::<Account>("accounts")
            .update_one(
                doc! {"_id": ObjectId::parse_str(&self._id)?, "user_id": ObjectId::parse_str(&self.user_id)?},
                doc! {"$set": doc! {"name": &self.name, "amount": &self.amount, "debt": &self.debt}},
            )
            .await?)
    }

    pub async fn accounts_total_for(user_id: &ObjectId, client: &mongodb::Client) -> f64 {
        let collection: Collection<Account> =
            client.default_database().unwrap().collection("accounts");

        let mut accounts: Vec<Account> = Vec::new();
        match collection.find(doc! {"user_id": user_id}).await {
            Ok(mut cursor) => {
                while cursor.advance().await.unwrap() {
                    match cursor.deserialize_current() {
                        Ok(account) => {
                            accounts.push(account);
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

        let debt = accounts
            .iter()
            .filter(|a| a.debt)
            .map(|e| e.amount)
            .reduce(|memo, amount| memo + amount)
            .unwrap_or(0.0);
        let non_debt = accounts
            .iter()
            .filter(|a| !a.debt)
            .map(|e| e.amount)
            .reduce(|memo, amount| memo + amount)
            .unwrap_or(0.0);

        non_debt - debt
    }
}
