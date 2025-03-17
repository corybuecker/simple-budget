use anyhow::{Result, anyhow};
use chrono::{DateTime, Datelike, Days, Duration, Months, NaiveTime, TimeDelta, TimeZone, Utc};
use chrono_tz::Tz;

pub trait Times {
    fn now(&self) -> DateTime<Utc>;
}

pub struct TimeProvider;

#[derive(Clone, Copy)]
pub struct TimeUtilities {
    pub timezone: Tz,
}

impl TimeUtilities {
    pub fn length_of_month(self, time_provider: &impl Times) -> Result<TimeDelta> {
        let start_of_month = self.start_of_month(time_provider)?;
        let end_of_month = self.end_of_month(time_provider)?;
        Ok(end_of_month - start_of_month)
    }

    pub fn remaining_length_of_day(self, time_provider: &impl Times) -> Result<TimeDelta> {
        let now = time_provider.now().with_timezone(&self.timezone);
        let tomorrow = now
            .checked_add_days(Days::new(1))
            .ok_or(anyhow!("could not parse date"))?;
        let tomorrow_midnight = tomorrow
            .with_time(NaiveTime::from_hms_opt(0, 0, 0).ok_or(anyhow!("could not parse date"))?)
            .single()
            .ok_or(anyhow!("could not parse date"))?;
        let end_of_day = tomorrow_midnight
            .checked_sub_signed(Duration::seconds(1))
            .ok_or(anyhow!("could not parse date"))?;

        Ok(end_of_day - now)
    }

    pub fn remaining_length_of_month(self, time_provider: &impl Times) -> Result<TimeDelta> {
        let now = time_provider.now().with_timezone(&self.timezone);
        let end_of_month = self.end_of_month(time_provider)?;
        Ok(end_of_month - now)
    }

    fn start_of_month(&self, time: &impl Times) -> Result<DateTime<Tz>> {
        let now = time.now().with_timezone(&self.timezone);

        let start_of_month = self
            .timezone
            .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0);

        start_of_month
            .single()
            .ok_or(anyhow!("error parsing datetime"))
    }

    fn end_of_month(&self, time: &impl Times) -> Result<DateTime<Tz>> {
        let start_of_month = self.start_of_month(time)?;
        let next_month = start_of_month
            .checked_add_months(Months::new(1))
            .ok_or(anyhow!("could not parse date"))?;

        let end_of_month = next_month
            .checked_sub_signed(Duration::seconds(1))
            .ok_or(anyhow!("could not parse date"))?;

        Ok(end_of_month)
    }
}

impl Times for TimeProvider {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{TimeDelta, TimeZone, Timelike, Utc};

    #[test]
    fn test_remaining_seconds_on_second_to_last_day() {
        struct MockTimeProvider;
        impl Times for MockTimeProvider {
            fn now(&self) -> chrono::DateTime<chrono::Utc> {
                Utc.with_ymd_and_hms(2024, 1, 30, 0, 0, 0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap()
            }
        }

        let time_utilities = TimeUtilities {
            timezone: chrono_tz::Tz::UTC,
        };
        let sec = time_utilities
            .remaining_length_of_month(&MockTimeProvider {})
            .unwrap();

        assert_eq!(sec, TimeDelta::seconds(172799))
    }

    #[test]
    fn test_remaining_seconds_on_last_day() {
        struct MockTimeProvider;
        impl Times for MockTimeProvider {
            fn now(&self) -> chrono::DateTime<chrono::Utc> {
                Utc.with_ymd_and_hms(2024, 1, 31, 0, 0, 0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap()
            }
        }

        let time_utilities = TimeUtilities {
            timezone: chrono_tz::Tz::UTC,
        };
        let sec = time_utilities
            .remaining_length_of_month(&MockTimeProvider {})
            .unwrap();

        assert_eq!(sec, TimeDelta::seconds(86399))
    }
}
