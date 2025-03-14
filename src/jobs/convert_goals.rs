use crate::{
    errors::AppError,
    models::{envelope::Envelope, goal::Goal},
};
use anyhow::{Result, anyhow};
use chrono::Utc;
use deadpool_postgres::Pool;
use tracing::info;

pub async fn convert_goals(pool: &Pool) -> Result<f64, AppError> {
    info!("converting goals to envelopes at {}", Utc::now());

    let mut manager = pool.get().await.unwrap();
    let transaction = manager.transaction().await?;
    let client = transaction.client();

    let goals = Goal::get_expired(client).await?;

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

        let mut goal = Goal {
            id: None,
            user_id: Some(user_id),
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
