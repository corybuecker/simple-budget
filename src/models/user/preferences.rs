use anyhow::{Result, anyhow};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GoalHeader {
    Accumulated,
    DaysRemaining,
    PerDay,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Preferences {
    pub timezone: Option<String>,
    pub goal_header: Option<GoalHeader>,
    pub forecast_offset: Option<i64>,
    pub monthly_income: Option<Decimal>,
    pub accelerate_goals: Option<bool>,
    pub accelerate_non_monthly: Option<bool>,
}

impl Preferences {
    pub fn default() -> Self {
        Self {
            timezone: Some("UTC".to_owned()),
            goal_header: Some(GoalHeader::Accumulated),
            forecast_offset: Some(1),
            monthly_income: Some(Decimal::ZERO),
            accelerate_goals: Some(false),
            accelerate_non_monthly: None,
        }
    }

    pub fn timezone(&self) -> Result<String> {
        self.timezone
            .clone()
            .or(Some("UTC".to_owned()))
            .ok_or(anyhow!("failure fetching timezone"))
    }

    pub fn monthly_income(&self) -> Result<Decimal> {
        self.monthly_income
            .or(Some(Decimal::ZERO))
            .ok_or(anyhow!("failure fetching monthly income"))
    }

    pub fn accelerate_goals(&self) -> Result<bool> {
        self.accelerate_goals
            .or(Some(false))
            .ok_or(anyhow!("failure fetching monthly income"))
    }

    pub fn accelerate_non_monthly(&self) -> Result<bool> {
        self.accelerate_non_monthly
            .or(Some(false))
            .ok_or(anyhow!("failure fetching monthly income"))
    }
}
