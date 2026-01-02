use crate::{
    errors::AppError,
    models::{envelope::Envelope, goal::Goal, user::User},
    utilities::dates::{TimeUtilities, Times},
};
use anyhow::{Result, anyhow};
use chrono::Utc;
use chrono_tz::Tz;
use rust_database_common::{DatabasePool, GenericClient};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use std::str::FromStr;
use tracing::info;

pub async fn convert_goals(pool: &DatabasePool, time: &impl Times) -> Result<f64, AppError> {
    let mut manager = pool.get_client().await?;
    let transaction = manager.transaction().await?;

    let results = private_convert_goals(&transaction, time).await?;

    transaction.commit().await?;

    Ok(results)
}

async fn private_convert_goals(
    client: &impl GenericClient,
    time: &impl Times,
) -> Result<f64, AppError> {
    info!("converting goals to envelopes at {}", Utc::now());

    let goals = Goal::get_expired(client, time.now()).await?;
    for goal in goals {
        let envelope = Envelope {
            id: None,
            name: goal.name.clone(),
            amount: goal.target,
            user_id: goal.user_id,
        };

        envelope.create(client).await?;
        let new_goal = goal.increment()?;
        new_goal.update(client).await?;
    }

    let goals = Goal::get_all_unscoped(client).await?;
    for goal in goals {
        goal.accumulate(client, time).await?;
    }

    let goals = Goal::get_all_unscoped(client).await?;
    for goal in goals {
        let user = User::get_by_id(client, goal.user_id).await?;
        let timezone = user.timezone()?;
        let time_utilities = TimeUtilities {
            timezone: Tz::from_str(&timezone)?,
        };

        let length_of_month = time_utilities.length_of_month(time)?;
        let length_of_month_in_seconds = Decimal::from_i64(length_of_month.num_seconds())
            .ok_or(anyhow!("could not convert remaining seconds to decimal"))?;

        info!(
            "🚧 length_of_month_in_seconds -> {:#?}",
            length_of_month_in_seconds
        );

        let monthly_income = user.monthly_income()?;
        let spendable_per_second = monthly_income / length_of_month_in_seconds;
        info!("🚧 spendable_per_second -> {:#?}", spendable_per_second);

        let remaining_length_of_month = time_utilities.remaining_length_of_month(time)?;
        let remaining_length_of_month_in_seconds =
            Decimal::from_i64(remaining_length_of_month.num_seconds())
                .ok_or(anyhow!("could not convert remaining seconds to decimal"))?;

        info!(
            "🚧 remaining_length_of_month_in_seconds -> {:#?}",
            remaining_length_of_month_in_seconds
        );
        let remaining_spendable = user.total_balance(client).await?;
        info!("🚧 remaining_spendable -> {:#?}", remaining_spendable);

        let remaining_spendable_per_second =
            remaining_spendable / remaining_length_of_month_in_seconds;

        info!(
            "🚧 remaining_spendable_per_second -> {:#?}",
            remaining_spendable_per_second
        );
        let acceleration_amount_per_second = Decimal::max(
            Decimal::ZERO,
            remaining_spendable_per_second - spendable_per_second,
        );
        let acceleration_amount =
            acceleration_amount_per_second * remaining_length_of_month_in_seconds;

        info!("🚧 acceleration_amount -> {:#?}", acceleration_amount);
        goal.accelerate(client, acceleration_amount).await?;
    }

    Ok(1.0)
}

#[cfg(test)]
mod tests {
    use crate::database_pool;
    use crate::jobs::convert_goals::private_convert_goals;
    use crate::models::account::Account;
    use crate::models::envelope::Envelope;
    use crate::models::goal::{Goal, Recurrence};
    use crate::models::user::{Preferences, User};
    use crate::test_utils::user_for_tests;
    use crate::utilities::dates::Times;
    use chrono::{Days, Duration, TimeZone, Timelike, Utc};
    use postgres_types::Json;
    use rust_database_common::DatabasePool;
    use rust_database_common::GenericClient;
    use rust_decimal::Decimal;
    use std::ops::Sub;

    struct MockTimeProvider;
    impl Times for MockTimeProvider {
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap()
        }
    }

    async fn setup() -> (User, DatabasePool, MockTimeProvider, Goal) {
        let database_pool = database_pool(Some(
            "postgres://simple_budget@localhost:5432/simple_budget_test",
        ))
        .await
        .unwrap();
        let time = MockTimeProvider {};

        let client = &database_pool.get_client().await.unwrap();
        let user = user_for_tests(client, None).await.unwrap();

        let goal = Goal {
            id: None,
            user_id: user.id,
            accumulated_amount: Decimal::ZERO,
            name: "convert_goals".to_owned(),
            target_date: time.now().sub(Duration::days(2)),
            target: Decimal::new(70, 0),
            recurrence: Recurrence::Weekly,
            start_date: None,
        };

        let goal = goal.create(client).await.unwrap();

        (
            user.clone(),
            database_pool.clone(),
            MockTimeProvider {},
            goal.clone(),
        )
    }

    // These tests are prone to database deadlocks due to the nature of the tests.
    // Running them sequentially can help avoid deadlocks.
    #[tokio::test]
    async fn test_convert_goalsrunner() {
        test_accelerate_goal().await;
        test_accumulate_goal().await;
        test_convert_goal_to_envelope().await;

        let mut database_pool = database_pool(Some(
            "postgres://simple_budget@localhost:5432/simple_budget_test",
        ))
        .await
        .unwrap();

        database_pool.connect().await.unwrap();
        database_pool
            .get_client()
            .await
            .unwrap()
            .execute("TRUNCATE TABLE goals", &[])
            .await
            .unwrap();
    }

    async fn test_accelerate_goal() {
        let (user, pool, time, goal) = setup().await;

        let mut client = pool.get_client().await.unwrap();
        let transaction = client.transaction().await.unwrap();

        let account = Account {
            user_id: user.id,
            id: None,
            name: "test".to_string(),
            amount: Decimal::new(100, 0),
            debt: false,
        };

        account.create(&transaction).await.unwrap();

        let mut goal = goal.clone();
        goal.target_date = time.now().checked_add_days(Days::new(3)).unwrap();

        goal.update(&transaction).await.unwrap();

        let mut user = user.clone();
        let mut preferences = Preferences::default();
        preferences.monthly_income = Some(Decimal::new(3100, 0));
        user.preferences = Some(Json(preferences));
        let user = user.update(&transaction).await.unwrap();

        private_convert_goals(&transaction, &time).await.unwrap();

        let goal: Goal = transaction
            .query_one(
                "SELECT * FROM goals WHERE user_id = $1 LIMIT 1",
                &[&user.id],
            )
            .await
            .unwrap()
            .try_into()
            .unwrap();

        assert!(goal.accumulated_amount - Decimal::new(4516, 2) < Decimal::new(1, 2));

        transaction.rollback().await.unwrap();
    }

    async fn test_accumulate_goal() {
        let (user, pool, time, _) = setup().await;

        let mut client = pool.get_client().await.unwrap();
        let transaction = client.transaction().await.unwrap();

        private_convert_goals(&transaction, &time).await.unwrap();

        let goal: Goal = transaction
            .query_one(
                "SELECT * FROM goals WHERE user_id = $1 LIMIT 1",
                &[&user.id],
            )
            .await
            .unwrap()
            .try_into()
            .unwrap();

        assert!(goal.accumulated_amount - Decimal::new(20, 0) < Decimal::new(1, 5));

        transaction.rollback().await.unwrap();
    }

    async fn test_convert_goal_to_envelope() {
        let (user, pool, time, _) = setup().await;

        let mut client = pool.get_client().await.unwrap();
        let transaction = client.transaction().await.unwrap();

        private_convert_goals(&transaction, &time).await.unwrap();

        let envelope = transaction
            .query_one(
                "SELECT * FROM envelopes WHERE user_id = $1 LIMIT 1",
                &[&user.id],
            )
            .await
            .unwrap();

        let envelope: Envelope = envelope.try_into().unwrap();

        assert_eq!(envelope.amount, Decimal::new(70, 0));

        let goal: Goal = transaction
            .query_one(
                "SELECT * FROM goals WHERE user_id = $1 LIMIT 1",
                &[&user.id],
            )
            .await
            .unwrap()
            .try_into()
            .unwrap();

        assert!(goal.target_date > time.now());

        transaction.rollback().await.unwrap();
    }
}
