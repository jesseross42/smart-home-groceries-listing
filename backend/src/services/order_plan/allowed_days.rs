//! The weekdays the household can take a delivery on (issue #74).
//!
//! Pure: the allowed days from Settings › Delivery in, the reported fees on
//! those days and whether the delivery asked for can fall on one. No allowed
//! days saved means every day is allowed.

use chrono::{Datelike, Duration, NaiveDate};

use crate::models::delivery_rows::DeliveryDay;
use crate::services::delivery_day_fees::{weekday, DayFee, MAX_DAYS_AHEAD};

/// Why the delivery asked for cannot come on an allowed day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayProblem {
    /// The weekday asked for (or the usual day) is not an allowed day.
    DayNotAllowed,
    /// No allowed day falls from today to the last day asked for.
    NoAllowedDayInRange,
}

/// The fees on allowed days only.
pub fn only_allowed(fees: &[DayFee], allowed: &[DeliveryDay]) -> Vec<DayFee> {
    fees.iter()
        .filter(|fee| is_allowed(allowed, fee.date))
        .copied()
        .collect()
}

/// What stops a delivery on `day` (a weekday asked for), or within
/// `days_ahead` days from `today` (0 = today only), from being allowed.
pub fn problem(
    allowed: &[DeliveryDay],
    day: Option<DeliveryDay>,
    today: NaiveDate,
    days_ahead: Option<u8>,
) -> Option<DayProblem> {
    if allowed.is_empty() {
        return None;
    }
    if let Some(day) = day {
        return (!allowed.contains(&day)).then_some(DayProblem::DayNotAllowed);
    }
    let last = i64::from(days_ahead?.min(MAX_DAYS_AHEAD));
    let reachable = (0..=last).any(|ahead| is_allowed(allowed, today + Duration::days(ahead)));
    (!reachable).then_some(DayProblem::NoAllowedDayInRange)
}

/// True when a delivery may come on `date`.
fn is_allowed(allowed: &[DeliveryDay], date: NaiveDate) -> bool {
    allowed.is_empty() || allowed.iter().any(|day| weekday(*day) == date.weekday())
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::services::stores::Store;

    /// A Monday.
    fn monday() -> NaiveDate {
        "2026-11-16".parse().unwrap()
    }

    fn fee(date: &str) -> DayFee {
        DayFee {
            store: Store::Woolworths,
            date: date.parse().unwrap(),
            fee: dec!(5),
        }
    }

    #[test]
    fn no_allowed_days_means_every_day_is_allowed() {
        let fees = [fee("2026-11-16"), fee("2026-11-17")];
        assert_eq!(only_allowed(&fees, &[]), fees.to_vec());
        assert_eq!(
            problem(&[], Some(DeliveryDay::Sunday), monday(), None),
            None
        );
        assert_eq!(problem(&[], None, monday(), Some(0)), None);
    }

    #[test]
    fn fees_on_other_days_are_dropped() {
        let fees = [fee("2026-11-16"), fee("2026-11-17"), fee("2026-11-24")];
        let kept = only_allowed(&fees, &[DeliveryDay::Tuesday]);
        assert_eq!(kept, vec![fee("2026-11-17"), fee("2026-11-24")]);
    }

    #[test]
    fn a_weekday_asked_for_must_be_allowed() {
        let allowed = [DeliveryDay::Tuesday, DeliveryDay::Saturday];
        assert_eq!(
            problem(&allowed, Some(DeliveryDay::Friday), monday(), None),
            Some(DayProblem::DayNotAllowed)
        );
        assert_eq!(
            problem(&allowed, Some(DeliveryDay::Saturday), monday(), None),
            None
        );
    }

    #[test]
    fn a_range_must_reach_an_allowed_day_and_the_last_day_counts() {
        let allowed = [DeliveryDay::Thursday];
        // Monday + 2 = Wednesday: one day short.
        assert_eq!(
            problem(&allowed, None, monday(), Some(2)),
            Some(DayProblem::NoAllowedDayInRange)
        );
        // Monday + 3 = Thursday: the last day is allowed.
        assert_eq!(problem(&allowed, None, monday(), Some(3)), None);
        // Today counts too.
        assert_eq!(
            problem(&[DeliveryDay::Monday], None, monday(), Some(0)),
            None
        );
    }

    #[test]
    fn any_day_with_no_range_always_has_an_allowed_day() {
        assert_eq!(problem(&[DeliveryDay::Sunday], None, monday(), None), None);
    }
}
