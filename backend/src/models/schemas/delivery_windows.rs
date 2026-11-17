//! The store-tab request that refreshes the planner's delivery windows.

use chrono::{Duration, TimeZone};
use serde::Deserialize;
use validator::{Validate, ValidationError, ValidationErrors};

use super::trolley_handoffs::MAX_SECRET_LEN;
use crate::services::delivery_day_fees::STORE_ZONE;
use crate::services::delivery_windows::DeliveryWindow;
use crate::services::stores::Store;

/// A snapshot reported by the store's logged-in browser tab.
#[derive(Deserialize, Validate)]
pub struct DeliveryWindowsReport {
    #[validate(length(max = MAX_SECRET_LEN))]
    pub secret: String,
    pub store: Store,
    #[validate(length(max = 1000), nested)]
    pub windows: Vec<DeliveryWindow>,
}

impl Validate for DeliveryWindow {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let valid_times = STORE_ZONE
            .from_local_datetime(&self.start)
            .single()
            .zip(STORE_ZONE.from_local_datetime(&self.end).single())
            .is_some_and(|(start, end)| {
                end > start
                    && end - start <= Duration::days(1)
                    && (self.end.date() - self.start.date()).num_days() <= 1
            });
        if valid_times && !self.fee.is_sign_negative() {
            return Ok(());
        }
        let mut errors = ValidationErrors::new();
        errors.add("window", ValidationError::new("invalid_delivery_window"));
        Err(errors)
    }
}
