use std::str::FromStr;

use crate::{
    errors::AppError,
    models::{envelope::Envelope, goal::Goal, user::User},
    utilities::dates::{TimeProvider, TimeUtilities},
};
use anyhow::{Result, anyhow};
use chrono::Utc;
use chrono_tz::Tz;
use deadpool_postgres::Pool;
use rust_decimal::{Decimal, prelude::FromPrimitive};
use tracing::info;

pub async fn convert_goals(pool: &Pool) -> Result<f64, AppError> {
    info!("converting goals to envelopes at {}", Utc::now());

    let mut manager = pool.get().await.unwrap();
    let transaction = manager.transaction().await?;
    let client = transaction.client();
    let time_provider = &TimeProvider {};

    let goals = Goal::get_all_unscoped(client).await?;
    for goal in goals {
        let user = User::get_by_id(client, goal.user_id).await?;
        let monthly_income = user.monthly_income()?;
        let remaining_spendable = user.total_balance(client).await?;

        let timezone = user.timezone()?;
        let time_utilities = TimeUtilities {
            timezone: Tz::from_str(&timezone)?,
        };
        let remaining_seconds = time_utilities.remaining_length_of_month(time_provider)?;
        let remaining_seconds = Decimal::from_i64(remaining_seconds.num_seconds())
            .ok_or(anyhow!("could not convert remaining seconds to decimal"))?;
        let length_of_month = time_utilities.length_of_month(time_provider)?;
        let length_of_month = Decimal::from_i64(length_of_month.num_seconds())
            .ok_or(anyhow!("could not convert remaining seconds to decimal"))?;
        let spendable_per_second = monthly_income / length_of_month;
        let remaining_spendable_per_second = remaining_spendable / remaining_seconds;
        let acceleration_amount = Decimal::max(
            Decimal::ZERO,
            remaining_spendable_per_second - spendable_per_second,
        ) * remaining_seconds;

        let goal = goal.accelerate(acceleration_amount)?;
        goal.accumulate(client, time_provider).await?;
    }

    let goals = Goal::get_expired(client).await?;
    for goal in goals {
        let envelope = Envelope {
            id: None,
            name: goal.name.clone(),
            amount: goal.target,
            user_id: goal.user_id,
        };

        envelope.create(client).await?;
        let new_goal = goal.increment();
        let _new_goal = new_goal.update(client).await?;
    }

    transaction.commit().await?;

    Ok(1.0)
}

#[cfg(test)]
mod tests {
    use crate::models::goal::{Goal, Recurrence};
    use crate::test_utils::state_for_tests;
    use crate::{jobs::convert_goals::convert_goals, models::envelope::Envelope};
    use chrono::{Duration, Utc};
    use rust_decimal::Decimal;
    use std::ops::Sub;
    use tokio_postgres::GenericClient;

    #[tokio::test]
    async fn test_convert_goals() {
        let (shared_state, user_extension) = state_for_tests().await.unwrap();
        let user_id = user_extension.0.id;
        let manager = shared_state.pool.get().await.unwrap();
        let client = manager.client();

        let goal = Goal {
            id: None,
            user_id,
            accumulated_amount: Decimal::ZERO,
            name: "convert_goals".to_owned(),
            target_date: Utc::now().sub(Duration::days(2)),
            target: Decimal::new(100, 0),
            recurrence: Recurrence::Weekly,
        };

        goal.create(client).await.unwrap();

        convert_goals(&shared_state.pool).await.unwrap();

        let envelope = client
            .query_one(
                "SELECT * FROM envelopes WHERE user_id = $1 LIMIT 1",
                &[&user_id],
            )
            .await
            .unwrap();

        let envelope: Envelope = envelope.try_into().unwrap();

        assert_eq!(envelope.amount, Decimal::new(100, 0));

        let goal: Goal = client
            .query_one(
                "SELECT * FROM goals WHERE user_id = $1 LIMIT 1",
                &[&user_id],
            )
            .await
            .unwrap()
            .try_into()
            .unwrap();

        assert!(goal.target_date > Utc::now());
    }
}
