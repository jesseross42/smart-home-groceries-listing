//! The cheapest delivery fee each store showed for each day (issue #70).
//!
//! The "Fill trolley" bookmark reports them after it reads the store's
//! delivery times ([`DeliveryDayFeeService::record`]). The order planner then
//! charges the cheapest fee on the household's delivery day, or on any day
//! when none is set ([`cheapest`]), instead of the typed fee. A store with no
//! report for that day keeps the fee typed in Settings › Delivery.

mod repository;

use chrono::{Datelike, NaiveDate, Utc, Weekday};
use chrono_tz::Tz;
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::error::ApiError;
use crate::models::delivery_rows::DeliveryDay;
use crate::services::stores::Store;

/// The stores' days are Australian east-coast days.
const STORE_ZONE: Tz = chrono_tz::Australia::Sydney;

/// How many days one report may carry; the stores show about a week.
pub const MAX_REPORTED_DAYS: u64 = 21;

/// One store's cheapest fee on one day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::FromRow)]
pub struct DayFee {
    pub store: Store,
    #[sqlx(rename = "delivery_date")]
    pub date: NaiveDate,
    #[sqlx(rename = "cheapest_fee")]
    pub fee: Decimal,
}

/// Reads and saves the reported fees.
pub struct DeliveryDayFeeService<'a> {
    pool: &'a PgPool,
}

impl<'a> DeliveryDayFeeService<'a> {
    /// Borrows the pool for one request.
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Saves what the bookmark saw at `store`, replacing earlier reports for
    /// the same days.
    pub async fn record(
        &self,
        store: Store,
        days: &[(NaiveDate, Decimal)],
    ) -> Result<(), ApiError> {
        let mut tx = self.pool.begin().await?;
        for (date, fee) in days.iter().take(MAX_REPORTED_DAYS as usize) {
            repository::upsert(&mut *tx, store, *date, *fee).await?;
        }
        tx.commit().await?;
        tracing::info!(store = ?store, days = days.len(), "delivery day fees recorded");
        Ok(())
    }

    /// Every store's fees from today on, earliest first.
    pub async fn upcoming(&self) -> Result<Vec<DayFee>, ApiError> {
        repository::from(self.pool, store_today()).await
    }
}

/// Today in the stores' time zone.
pub fn store_today() -> NaiveDate {
    Utc::now().with_timezone(&STORE_ZONE).date_naive()
}

/// The fee to charge at `store`: on the first coming `day` the store
/// reported, or the cheapest of every reported day when `day` is `None`.
/// `fees` must be earliest first. `None` when nothing fits.
pub fn cheapest(fees: &[DayFee], store: Store, day: Option<DeliveryDay>) -> Option<DayFee> {
    let at_store = fees.iter().filter(|fee| fee.store == store);
    match day {
        Some(day) => at_store
            .filter(|fee| fee.date.weekday() == weekday(day))
            .copied()
            .next(),
        None => at_store.copied().min_by_key(|fee| (fee.fee, fee.date)),
    }
}

/// The chrono weekday for a delivery day.
fn weekday(day: DeliveryDay) -> Weekday {
    match day {
        DeliveryDay::Monday => Weekday::Mon,
        DeliveryDay::Tuesday => Weekday::Tue,
        DeliveryDay::Wednesday => Weekday::Wed,
        DeliveryDay::Thursday => Weekday::Thu,
        DeliveryDay::Friday => Weekday::Fri,
        DeliveryDay::Saturday => Weekday::Sat,
        DeliveryDay::Sunday => Weekday::Sun,
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    fn fee(store: Store, date: &str, fee: Decimal) -> DayFee {
        DayFee {
            store,
            date: date.parse().unwrap(),
            fee,
        }
    }

    /// Mon 17 Nov 2026 to Wed 25 Nov 2026.
    fn reported() -> Vec<DayFee> {
        vec![
            fee(Store::Woolworths, "2026-11-17", dec!(2)),
            fee(Store::Coles, "2026-11-17", dec!(9)),
            fee(Store::Woolworths, "2026-11-21", dec!(15)),
            fee(Store::Woolworths, "2026-11-24", dec!(1)),
        ]
    }

    #[test]
    fn a_day_takes_the_first_coming_date_on_that_weekday() {
        let tuesday = cheapest(&reported(), Store::Woolworths, Some(DeliveryDay::Tuesday));
        assert_eq!(tuesday.unwrap().date.to_string(), "2026-11-17");
        let saturday = cheapest(&reported(), Store::Woolworths, Some(DeliveryDay::Saturday));
        assert_eq!(saturday.unwrap().fee, dec!(15));
    }

    #[test]
    fn no_day_takes_the_cheapest_day() {
        let any = cheapest(&reported(), Store::Woolworths, None).unwrap();
        assert_eq!(any.fee, dec!(1));
        assert_eq!(any.date.to_string(), "2026-11-24");
    }

    #[test]
    fn a_day_the_store_did_not_report_gives_nothing() {
        assert_eq!(
            cheapest(&reported(), Store::Coles, Some(DeliveryDay::Friday)),
            None
        );
        assert_eq!(cheapest(&[], Store::Coles, None), None);
    }
}
