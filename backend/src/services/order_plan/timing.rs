//! Selects the cheapest window that fits the household's delivery constraints.

use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};
use serde::Serialize;
use sqlx::PgPool;

use super::fees::{charge, DeliveryCharge};
use crate::error::ApiError;
use crate::models::delivery_rows::DeliveryDay;
use crate::services::delivery_day_fees::{weekday, STORE_ZONE};
use crate::services::delivery_settings::{DeliverySettings, FeeRules};
use crate::services::delivery_windows::{self, DeliveryWindow, WindowSnapshot};
use crate::services::stores::Store;
use rust_decimal::Decimal;

/// Why a store part cannot be scheduled under the household's constraints.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TimingProblem {
    /// The store has no recent report.
    WindowsUnavailable,
    /// The store reported, but no window satisfies all constraints.
    NoMatchingWindow,
    /// The configured deadline has passed.
    NeedByPassed,
}

/// The timing result for one store's part of an option.
#[derive(Debug, Clone, Copy)]
pub enum StoreTiming {
    Unconstrained,
    Scheduled(DeliveryWindow),
    Unavailable(TimingProblem),
}

impl StoreTiming {
    /// Whether this store part satisfies every timing constraint.
    pub fn fits(self) -> bool {
        !matches!(self, Self::Unavailable(_))
    }

    /// The delivery charge uses the matching window's fee when available.
    pub fn charge(self, mut rules: FeeRules, subtotal: Decimal) -> DeliveryCharge {
        if let Self::Scheduled(window) = self {
            rules.delivery_fee = Some(window.fee);
        }
        charge(&rules, subtotal)
    }
}

/// Available timing results with and without perishables at each store.
#[derive(Default)]
pub struct Timing {
    stores: Vec<(Store, [StoreTiming; 2])>,
}

impl Timing {
    /// Reads recent windows only when a need-by deadline is configured.
    pub async fn load(
        pool: &PgPool,
        settings: &DeliverySettings,
        delivery_day: Option<DeliveryDay>,
        within_days: Option<u8>,
        now: DateTime<Utc>,
    ) -> Result<Self, ApiError> {
        let Some(deadline) = settings.need_by else {
            return Ok(Self::default());
        };
        let deadline = STORE_ZONE
            .from_local_datetime(&deadline)
            .single()
            .map(|value| value.to_utc());
        let snapshots = delivery_windows::fresh(pool, now).await?;
        let last_day = within_days.map(|days| {
            now.with_timezone(&STORE_ZONE).date_naive() + Duration::days(i64::from(days))
        });
        let days = AllowedDates {
            weekday: delivery_day,
            last_day,
            allowed_weekdays: &settings.allowed_delivery_days,
        };
        let stores = Store::ALL
            .into_iter()
            .map(|store| {
                let snapshot = snapshots.iter().find(|snapshot| snapshot.store == store);
                let choose = |hours| choose_window(snapshot, &days, now, deadline, hours);
                (store, [choose(None), choose(settings.perishable_hours)])
            })
            .collect();
        Ok(Self { stores })
    }

    /// The timing result if this store delivers the given type of basket.
    pub fn at(&self, store: Store, has_perishables: bool) -> StoreTiming {
        self.stores
            .iter()
            .find(|(at, _)| *at == store)
            .map_or(StoreTiming::Unconstrained, |(_, timing)| {
                timing[usize::from(has_perishables)]
            })
    }
}

/// The calendar filters applied to both ends of a delivery window.
struct AllowedDates<'a> {
    weekday: Option<DeliveryDay>,
    last_day: Option<NaiveDate>,
    allowed_weekdays: &'a [DeliveryDay],
}

impl AllowedDates<'_> {
    fn includes(&self, date: NaiveDate) -> bool {
        self.weekday
            .is_none_or(|wanted| date.weekday() == weekday(wanted))
            && self.last_day.is_none_or(|last| date <= last)
            && (self.allowed_weekdays.is_empty()
                || self
                    .allowed_weekdays
                    .iter()
                    .any(|day| weekday(*day) == date.weekday()))
    }
}

fn choose_window(
    snapshot: Option<&WindowSnapshot>,
    days: &AllowedDates<'_>,
    now: DateTime<Utc>,
    deadline: Option<DateTime<Utc>>,
    hours: Option<i16>,
) -> StoreTiming {
    let Some(deadline) = deadline.filter(|deadline| *deadline > now) else {
        return StoreTiming::Unavailable(TimingProblem::NeedByPassed);
    };
    let Some(snapshot) = snapshot else {
        return StoreTiming::Unavailable(TimingProblem::WindowsUnavailable);
    };
    let earliest = hours
        .and_then(|hours| deadline.checked_sub_signed(Duration::hours(i64::from(hours))))
        .map_or(now, |earliest| earliest.max(now));
    snapshot
        .windows
        .iter()
        .copied()
        .filter(|window| {
            days.includes(window.start.date())
                && days.includes(window.end.date())
                && STORE_ZONE
                    .from_local_datetime(&window.start)
                    .single()
                    .is_some_and(|start| start >= earliest)
                && STORE_ZONE
                    .from_local_datetime(&window.end)
                    .single()
                    .is_some_and(|end| end <= deadline)
        })
        .min_by_key(|window| (window.fee, window.start, window.end))
        .map_or(
            StoreTiming::Unavailable(TimingProblem::NoMatchingWindow),
            StoreTiming::Scheduled,
        )
}
