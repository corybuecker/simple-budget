use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use tracing::info;

use crate::{
    database_client,
    models::{envelope::Envelope, goal::Goal},
};

pub async fn convert_goals() -> Result<f64> {
    info!("converting goals to envelopes at {}", Utc::now());

    let mut client = database_client().await?;
    let transaction = client.transaction().await?;

    let goals = Goal::get_expired(transaction.client())
        .await
        .context("convert goals")?;

    for goal in goals {
        let envelope = Envelope {
            id: None,
            name: goal.name.clone(),
            amount: goal.target,
            user_id: Some(
                goal.user_id
                    .ok_or(anyhow!("envelope must have a user ID"))?,
            ),
        };

        envelope.create(transaction.client()).await?;

        let new_goal = goal.increment();

        new_goal.update(transaction.client()).await?;
    }
    Ok(1.0)
}

//#[cfg(test)]
//mod tests {
//    use crate::{jobs::convert_goals::convert_goals, models::envelope::Envelope};
//    use bson::{doc, oid::ObjectId};
//    use chrono::{Duration, Utc};
//    use std::ops::Sub;
//
//    use crate::{
//        models::goal::{Goal, Recurrence},
//        mongo_client,
//    };
//    #[tokio::test]
//    async fn test_convert_goals() {
//        let client = mongo_client().await.unwrap();
//
//        let goals = client
//            .default_database()
//            .unwrap()
//            .collection::<Goal>("goals");
//
//        goals
//            .delete_many(doc! {"name": "convert_goals"})
//            .await
//            .unwrap();
//
//        let envelopes = client
//            .default_database()
//            .unwrap()
//            .collection::<Envelope>("envelopes");
//
//        envelopes
//            .delete_many(doc! {"name": "convert_goals"})
//            .await
//            .unwrap();
//
//        let _ = goals
//            .insert_one(Goal {
//                name: "convert_goals".to_owned(),
//                target_date: Utc::now().sub(Duration::seconds(100)),
//                recurrence: Recurrence::Daily,
//                user_id: ObjectId::new().to_hex(),
//                _id: ObjectId::new().to_hex(),
//                target: 100.0,
//            })
//            .await
//            .unwrap();
//
//        match convert_goals().await {
//            Ok(result) => println!("{}", result),
//            Err(error) => println!("conversion error: {}", error),
//        };
//
//        let envelope = envelopes
//            .find_one(doc! {"name": "convert_goals"})
//            .await
//            .expect("error fetching envelope")
//            .expect("could not find envelope");
//
//        assert_eq!(envelope.amount, 100.0);
//
//        let goal = goals
//            .find_one(doc! {"name": "convert_goals"})
//            .await
//            .expect("error fetching goal")
//            .expect("could not find goal");
//        println!("{:#?}", goal);
//        assert!(goal.target_date > Utc::now());
//    }
//}
