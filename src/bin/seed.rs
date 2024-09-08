use bson::{doc, oid::ObjectId};
use chrono::{Duration, Utc};
use rand::{thread_rng, Rng};
use simple_budget::models::{account::Account, envelope::Envelope, goal::Goal, user::User};

#[tokio::main]
async fn main() {
    let client = mongodb::Client::with_uri_str(
        "mongodb://localhost:27017/simple_budget?directConnection=true",
    )
    .await
    .expect("cannot connect to database");

    let accounts = client
        .default_database()
        .expect("cannot find database")
        .collection::<Account>("accounts");

    let envelopes = client
        .default_database()
        .expect("cannot find database")
        .collection::<Envelope>("envelopes");

    let goals = client
        .default_database()
        .expect("cannot find database")
        .collection::<Goal>("goals");

    let users = client
        .default_database()
        .expect("cannot find database")
        .collection::<User>("users");

    let user = users
        .find_one(doc! {})
        .await
        .expect("must manually create user")
        .expect("could not find user");

    let _ = accounts.drop().await;
    let _ = envelopes.drop().await;
    let _ = goals.drop().await;

    let mut account_seeds = Vec::<Account>::new();
    let mut goal_seeds = Vec::<Goal>::new();

    for i in 0..15 {
        account_seeds.push(account_generator(i + 1, user._id.clone()))
    }
    for i in 0..5 {
        goal_seeds.push(goal_generator(i + 1, user._id.clone()))
    }

    let _ = accounts.insert_many(account_seeds).await;
    let _ = goals.insert_many(goal_seeds).await;
}

fn goal_generator(index: u32, user_id: String) -> Goal {
    let mut trng = thread_rng();
    let rnd: f64 = trng.gen();

    Goal {
        _id: ObjectId::new().to_string(),
        user_id,
        name: format!("Test goal {}", index),
        target_date: Utc::now() + Duration::days(1),
        target: 10.0 * rnd,
        recurrence: simple_budget::models::goal::Recurrence::Daily,
    }
}
fn account_generator(index: u32, user_id: String) -> Account {
    let mut trng = thread_rng();
    let rnd: f64 = trng.gen();

    Account {
        _id: ObjectId::new().to_string(),
        user_id,
        name: format!("Test account {}", index),
        amount: (rnd * 1000.0).floor(),
        debt: index % 3 == 0,
    }
}
