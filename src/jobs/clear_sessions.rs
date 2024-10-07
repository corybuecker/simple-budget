use crate::{models::user::User, mongo_client};
use bson::doc;
use chrono::Utc;
use tracing::{error, info};

pub async fn clear_sessions() {
    info!("clearing old sessions at {}", Utc::now());
    let mongo = mongo_client().await.unwrap();

    let update_result = mongo
        .default_database()
        .unwrap()
        .collection::<User>("users")
        .update_many(
            doc! {},
            doc! {"$pull": doc! {"sessions": doc! {"expiration": doc! { "$lte": Utc::now()}}}},
        )
        .await;

    match update_result {
        Ok(_) => {}
        Err(err) => {
            error!("{}", err);
        }
    }
}
