use anyhow::Result;
use tokio_postgres::Client;

#[derive(Debug)]
pub struct Envelope {
    pub id: Option<i32>,
    pub user_id: i32,
    pub name: String,
    pub amount: f64,
}

impl Envelope {
    pub async fn create(&self, client: &Client) -> Result<()> {
        client
            .query(
                "INSERT INTO envelopes (user_id, name, amount) VALUES ($1, $2, $3)",
                &[&self.user_id, &self.name, &self.amount],
            )
            .await?;
        Ok(())
    }

    pub async fn update(&self, client: &Client) -> Result<()> {
        client
            .query(
                "UPDATE envelopes SET name = $1, amount = $2 WHERE id = $3 AND user_id = $4",
                &[&self.name, &self.amount, &self.id, &self.user_id],
            )
            .await?;
        Ok(())
    }
}

pub async fn envelopes_total_for(user_id: i32, client: &Client) -> Result<f64> {
    let total: f64 = client
        .query_one(
            "SELECT SUM(amount) AS sum FROM envelopes WHERE user_id = $1",
            &[&user_id],
        )
        .await?
        .try_get("sum")?;

    Ok(total)
}
