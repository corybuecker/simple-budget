use crate::models::{envelope::Envelope, goal::Goal};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use mongodb::ClientSession;
use std::str::FromStr;

pub async fn convert_goals(
    mut session: ClientSession,
    database: Option<String>,
) -> Result<f64, mongodb::error::Error> {
    session.start_transaction().await?;

    let database = match database {
        Some(database) => database,
        None => "simple_budget".to_owned(),
    };

    let envelopes = session
        .client()
        .database(&database)
        .collection::<Envelope>("envelopes");

    let goals = session
        .client()
        .database(&database)
        .collection::<Goal>("goals");

    let goal = goals
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

            let _ = goals
                .replace_one(
                    doc! {"_id": ObjectId::from_str(&goal._id).unwrap()},
                    new_goal,
                )
                .session(&mut session)
                .await?;

            session.commit_transaction().await?;

            Ok(1.0)
        }
        None => Ok(1.9),
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
            .database("simple_budget_test")
            .collection::<Goal>("goals");

        let envelopes = client
            .database("simple_budget_test")
            .collection::<Envelope>("envelopes");

        let _ = goals.delete_many(doc! {}).await;
        let _ = envelopes.delete_many(doc! {}).await;

        let _ = goals
            .insert_one(Goal {
                name: "test".to_owned(),
                target_date: Utc::now().sub(Duration::seconds(100)),
                recurrence: Recurrence::Daily,
                user_id: ObjectId::new().to_hex(),
                _id: ObjectId::new().to_hex(),
                target: 100.0,
            })
            .await
            .unwrap();

        let session = client.start_session().await.unwrap();

        match convert_goals(session, Some("simple_budget_test".to_owned())).await {
            Ok(result) => println!("{}", result),
            Err(error) => println!("conversion error: {}", error),
        };

        let envelope = envelopes
            .find_one(doc! {"name": "test"})
            .await
            .expect("error fetching envelope")
            .expect("could not find envelope");

        assert_eq!(envelope.amount, 100.0);

        let goal = goals
            .find_one(doc! {"name": "test"})
            .await
            .expect("error fetching goal")
            .expect("could not find goal");
        println!("{:#?}", goal);
        assert!(goal.target_date > Utc::now());
    }
}
