use chrono::{DateTime, Datelike, Months, TimeDelta, Timelike, Utc};
use chrono_tz::Tz;
use std::ops::Sub;

pub trait Times {
    fn now(&self) -> DateTime<Utc>;
}

pub struct TimeProvider;

#[derive(Clone, Copy)]
pub struct TimeUtilities {
    pub timezone: Tz,
}

impl TimeUtilities {
    pub fn remaining_seconds(self, time_provider: &impl Times) -> TimeDelta {
        let now = time_provider.now().with_timezone(&self.timezone);
        let end_of_month = self
            .end_of_month(time_provider)
            .expect("could not determine end of month");
        let end_of_next_month = self
            .end_of_next_month(time_provider)
            .expect("could not determine end of next month");
        let days = end_of_month - now;

        if days.num_days() == 0 {
            end_of_next_month - now
        } else {
            end_of_month - now
        }
    }

    pub fn end_of_next_month(self, time_provider: &impl Times) -> Result<DateTime<Tz>, String> {
        let now = time_provider
            .now()
            .with_timezone(&self.timezone)
            .checked_add_months(Months::new(2))
            .expect("failed to build datetime");
        let now = now.with_hour(0).ok_or("could not set time");
        let now = now?.with_minute(0).ok_or("could not set time");
        let now = now?.with_second(0).ok_or("could not set time");
        let now = now?.with_day0(0).ok_or("could not set day to zero");
        let now = now?.sub(TimeDelta::new(1, 0).unwrap());
        Ok(now)
    }

    pub fn end_of_month(self, time_provider: &impl Times) -> Result<DateTime<Tz>, String> {
        let now = time_provider
            .now()
            .with_timezone(&self.timezone)
            .checked_add_months(Months::new(1))
            .expect("failed to build datetime");
        let now = now.with_hour(0).ok_or("could not set time");
        let now = now?.with_minute(0).ok_or("could not set time");
        let now = now?.with_second(0).ok_or("could not set time");
        let now = now?.with_day0(0).ok_or("could not set day to zero");
        let now = now?.sub(TimeDelta::new(1, 0).unwrap());
        Ok(now)
    }
}

impl Times for TimeProvider {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[cfg(test)]
mod test {
    use super::{TimeUtilities, Times};
    use chrono::{TimeDelta, TimeZone, Timelike, Utc};
    struct MockTimeProvider;
    impl Times for MockTimeProvider {
        fn now(&self) -> chrono::DateTime<chrono::Utc> {
            Utc.with_ymd_and_hms(2024, 1, 30, 0, 0, 0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap()
        }
    }

    #[test]
    fn test_remaining_seconds() {
        let time_utilities = TimeUtilities {
            timezone: chrono_tz::Tz::UTC,
        };
        let sec = time_utilities.remaining_seconds(&MockTimeProvider {});

        assert_eq!(sec, TimeDelta::seconds(172799))
    }
}
