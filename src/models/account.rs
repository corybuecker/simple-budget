use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use tokio_postgres::Client;

#[derive(Deserialize, Serialize, Debug)]
pub struct Account {
    pub id: Option<i32>,
    pub user_id: Option<i32>,
    pub name: String,
    pub amount: f64,
    pub debt: bool,
}

impl TryInto<Account> for tokio_postgres::Row {
    type Error = anyhow::Error;

    fn try_into(self: tokio_postgres::Row) -> Result<Account> {
        Ok(Account {
            id: self.try_get("id")?,
            user_id: self.try_get("user_id")?,
            name: self.try_get("name")?,
            amount: self.try_get("amount")?,
            debt: self.try_get("debt")?,
        })
    }
}
impl Account {
    pub async fn create(&mut self, client: &Client) -> Result<()> {
        let row = client
            .query_one(
                "INSERT INTO accounts (user_id, name, amount, debt) VALUES ($1, $2, $3, $4) RETURNING id",
                &[&self.user_id, &self.name, &self.amount, &self.debt],
            )
            .await?;

        self.id = Some(row.try_get("id")?);
        Ok(())
    }

    pub async fn update(&self, client: &Client) -> Result<()> {
        client.query("UPDATE accounts SET name = $1, amount = $2, debt = $3 WHERE id = $4 AND user_id = $5", &[&self.name, &self.amount, &self.debt, &self.id, &self.user_id]).await?;
        Ok(())
    }

    pub async fn delete(&self, client: &Client) -> Result<()> {
        client
            .execute(
                "DELETE FROM accounts WHERE user_id = $1 and id = $2",
                &[&self.user_id, &self.id],
            )
            .await?;
        Ok(())
    }

    pub async fn get_one(client: &Client, id: i32, user_id: i32) -> Result<Self> {
        client
            .query_one(
                "SELECT accounts.* FROM accounts
                INNER JOIN users ON users.id = accounts.user_id
                WHERE users.id = $1 AND accounts.id = $2",
                &[&user_id, &id],
            )
            .await?
            .try_into()
    }

    pub async fn get_all(client: &Client, user_id: i32) -> Result<Vec<Self>> {
        let rows = client
            .query(
                "SELECT accounts.* FROM accounts INNER
            JOIN users ON users.id = accounts.user_id WHERE users.id = $1",
                &[&user_id],
            )
            .await?;

        let mut accounts = Vec::with_capacity(rows.len());
        for row in rows {
            accounts.push(row.try_into()?);
        }

        Ok(accounts)
    }

    pub async fn accounts_total_for(user_id: i32, client: &Client) -> f64 {
        let accounts = Self::get_all(client, user_id).await.unwrap();
        let debt = accounts
            .iter()
            .filter(|a| a.debt)
            .map(|e| e.amount)
            .reduce(|memo, amount| memo + amount)
            .unwrap_or(0.0);

        let non_debt = accounts
            .iter()
            .filter(|a| !a.debt)
            .map(|e| e.amount)
            .reduce(|memo, amount| memo + amount)
            .unwrap_or(0.0);

        non_debt - debt
    }
}
