use crate::errors::AppError;
use anyhow::Result;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde::Serialize;
use tokio_postgres::Client;

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct Account {
    pub id: Option<i32>,
    pub user_id: i32,
    pub name: String,
    pub amount: Decimal,
    pub debt: bool,
}

impl TryInto<Account> for tokio_postgres::Row {
    type Error = AppError;

    fn try_into(self: tokio_postgres::Row) -> Result<Account, AppError> {
        Ok(Account {
            id: self
                .try_get("id")
                .map_err(AppError::RecordDeserializationError)?,
            user_id: self
                .try_get("user_id")
                .map_err(AppError::RecordDeserializationError)?,
            name: self
                .try_get("name")
                .map_err(AppError::RecordDeserializationError)?,
            amount: self
                .try_get("amount")
                .map_err(AppError::RecordDeserializationError)?,
            debt: self
                .try_get("debt")
                .map_err(AppError::RecordDeserializationError)?,
        })
    }
}

impl Account {
    pub async fn create(&self, client: &Client) -> Result<Self, AppError> {
        let row = client
            .query_one(
                "INSERT INTO accounts (user_id, name, amount, debt) VALUES ($1, $2, $3, $4) RETURNING id",
                &[&self.user_id, &self.name, &self.amount, &self.debt],
            )
            .await?;

        let mut new_account = self.to_owned().clone();
        new_account.id = Some(row.try_get("id")?);
        Ok(new_account)
    }

    pub async fn update(&self, client: &Client) -> Result<(), AppError> {
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

    pub async fn get_one(client: &Client, id: i32, user_id: i32) -> Result<Self, AppError> {
        let row = client
            .query_one(
                "SELECT accounts.* FROM accounts
                INNER JOIN users ON users.id = accounts.user_id
                WHERE users.id = $1 AND accounts.id = $2",
                &[&user_id, &id],
            )
            .await
            .map_err(AppError::RecordNotFound)?;

        row.try_into()
    }

    pub async fn get_all(client: &Client, user_id: i32) -> Result<Vec<Self>, AppError> {
        let rows = client
            .query(
                "SELECT accounts.* FROM accounts INNER
            JOIN users ON users.id = accounts.user_id WHERE users.id = $1",
                &[&user_id],
            )
            .await
            .map_err(AppError::RecordNotFound)?;

        let mut accounts = Vec::with_capacity(rows.len());
        for row in rows {
            accounts.push(row.try_into()?);
        }

        Ok(accounts)
    }
}
