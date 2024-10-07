use crate::{
    models::{envelope::Envelope, goal::Goal},
    mongo_client,
};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use std::str::FromStr;
use tracing::info;

pub async fn convert_goals() -> Result<f64, mongodb::error::Error> {
    info!("converting goals to envelopes at {}", Utc::now());

    let mongo = mongo_client().await?;
    let mut session = mongo.start_session().await?;

    session.start_transaction().await?;

    let envelopes = session
        .client()
        .default_database()
        .unwrap()
        .collection::<Envelope>("envelopes");

    let goals_collection = session
        .client()
        .default_database()
        .unwrap()
        .collection::<Goal>("goals");

    let goal = goals_collection
        .find_one(
            doc! {"recurrence": doc! { "$ne": "never" }, "target_date": doc! {"$lt": Utc::now()}},
        )
        .session(&mut session)
        .await?;

    match goal {
        Some(goal) => {
            let envelope = Envelope {
                _id: ObjectId::new().to_hex(),
                name: goal.name.clone(),
                amount: goal.target,
                user_id: goal.user_id.clone(),
            };
            envelopes.insert_one(envelope).session(&mut session).await?;

            let new_goal = goal.increment();

            let _ = goals_collection
                .replace_one(
                    doc! {"_id": ObjectId::from_str(&goal._id).unwrap()},
                    new_goal,
                )
                .session(&mut session)
                .await?;

            session.commit_transaction().await?;

            Ok(1.0)
        }
        None => Ok(0.0),
    }
}

#[cfg(test)]
mod tests {
    use crate::{jobs::convert_goals::convert_goals, models::envelope::Envelope};
    use bson::{doc, oid::ObjectId};
    use chrono::{Duration, Utc};
    use std::ops::Sub;

    use crate::{
        models::goal::{Goal, Recurrence},
        mongo_client,
    };
    #[tokio::test]
    async fn test_convert_goals() {
        let client = mongo_client().await.unwrap();

        let goals = client
            .default_database()
            .unwrap()
            .collection::<Goal>("goals");

        goals
            .delete_many(doc! {"name": "convert_goals"})
            .await
            .unwrap();

        let envelopes = client
            .default_database()
            .unwrap()
            .collection::<Envelope>("envelopes");

        envelopes
            .delete_many(doc! {"name": "convert_goals"})
            .await
            .unwrap();

        let _ = goals
            .insert_one(Goal {
                name: "convert_goals".to_owned(),
                target_date: Utc::now().sub(Duration::seconds(100)),
                recurrence: Recurrence::Daily,
                user_id: ObjectId::new().to_hex(),
                _id: ObjectId::new().to_hex(),
                target: 100.0,
            })
            .await
            .unwrap();

        match convert_goals().await {
            Ok(result) => println!("{}", result),
            Err(error) => println!("conversion error: {}", error),
        };

        let envelope = envelopes
            .find_one(doc! {"name": "convert_goals"})
            .await
            .expect("error fetching envelope")
            .expect("could not find envelope");

        assert_eq!(envelope.amount, 100.0);

        let goal = goals
            .find_one(doc! {"name": "convert_goals"})
            .await
            .expect("error fetching goal")
            .expect("could not find goal");
        println!("{:#?}", goal);
        assert!(goal.target_date > Utc::now());
    }
}
