use anyhow::Result;
use serde::Serialize;
use tokio_postgres::Client;

#[derive(Serialize, Debug)]
pub struct Envelope {
    pub id: Option<i32>,
    pub user_id: Option<i32>,
    pub name: String,
    pub amount: f64,
}

impl TryInto<Envelope> for tokio_postgres::Row {
    type Error = anyhow::Error;

    fn try_into(self: tokio_postgres::Row) -> Result<Envelope> {
        Ok(Envelope {
            id: self.try_get("id")?,
            user_id: self.try_get("user_id")?,
            name: self.try_get("name")?,
            amount: self.try_get("amount")?,
        })
    }
}

impl Envelope {
    pub async fn get_one(client: &Client, id: i32, user_id: i32) -> Result<Self> {
        client
            .query_one(
                "SELECT envelopes.* FROM envelopes
                INNER JOIN users ON users.id = envelopes.user_id
                WHERE users.id = $1 AND envelopes.id = $2",
                &[&user_id, &id],
            )
            .await?
            .try_into()
    }

    pub async fn get_all(client: &Client, user_id: i32) -> Result<Vec<Self>> {
        let rows = client
            .query(
                "SELECT envelopes.* FROM envelopes INNER
            JOIN users ON users.id = envelopes.user_id WHERE users.id = $1",
                &[&user_id],
            )
            .await?;

        let mut envelopes = Vec::with_capacity(rows.len());
        for row in rows {
            envelopes.push(row.try_into()?);
        }

        Ok(envelopes)
    }

    pub async fn delete(&self, client: &Client) -> Result<()> {
        client
            .execute(
                "DELETE FROM envelopes WHERE user_id = $1 and id = $2",
                &[&self.user_id, &self.id],
            )
            .await?;
        Ok(())
    }

    pub async fn create(&mut self, client: &Client) -> Result<()> {
        let row = client
            .query_one(
                "INSERT INTO envelopes (user_id, name, amount) VALUES ($1, $2, $3) RETURNING id",
                &[&self.user_id, &self.name, &self.amount],
            )
            .await?;

        self.id = row.try_get("id")?;
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
