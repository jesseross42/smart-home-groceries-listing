//! Where the access-log file goes (`docs/features/FEATURE_ACCESS_LOGS.md`).
//!
//! Read on its own, before [`Settings`](super::Settings), because logging is
//! set up first so a bad setting can itself be logged.

use std::path::PathBuf;

use super::{optional, parse_or};
use crate::error::ConfigError;

/// Days an `access_logs` row is kept when `ACCESS_LOG_RETENTION_DAYS` is unset.
pub const DEFAULT_ACCESS_LOG_RETENTION_DAYS: u32 = 90;

/// The longest retention accepted: 100 years.
pub const MAX_ACCESS_LOG_RETENTION_DAYS: u32 = 36_500;

/// `ACCESS_LOG_DIR`: the folder for the rotating access-log files. Unset
/// means no file — the `access_logs` table is still written.
pub fn access_log_dir() -> Option<PathBuf> {
    optional("ACCESS_LOG_DIR").map(|dir| PathBuf::from(dir.trim()))
}

/// `ACCESS_LOG_RETENTION_DAYS`: days an `access_logs` row is kept before the
/// daily clean-up deletes it. `0` keeps rows forever; unset is
/// [`DEFAULT_ACCESS_LOG_RETENTION_DAYS`]. The files have their own 30-day limit.
///
/// More than [`MAX_ACCESS_LOG_RETENTION_DAYS`] is refused at startup: it is
/// almost certainly a typo, and PostgreSQL cannot subtract that many days.
pub(super) fn retention_days() -> Result<u32, ConfigError> {
    let days = parse_or(
        "ACCESS_LOG_RETENTION_DAYS",
        DEFAULT_ACCESS_LOG_RETENTION_DAYS,
    )?;
    check_retention_days(days)
}

/// `days`, or a configuration error when it is over the limit.
fn check_retention_days(days: u32) -> Result<u32, ConfigError> {
    if days > MAX_ACCESS_LOG_RETENTION_DAYS {
        return Err(ConfigError::Invalid {
            key: "ACCESS_LOG_RETENTION_DAYS".to_string(),
        });
    }
    Ok(days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retention_up_to_the_limit_is_accepted() {
        assert_eq!(check_retention_days(0).unwrap(), 0);
        assert_eq!(
            check_retention_days(MAX_ACCESS_LOG_RETENTION_DAYS).unwrap(),
            MAX_ACCESS_LOG_RETENTION_DAYS
        );
    }

    #[test]
    fn retention_over_the_limit_is_refused() {
        assert!(check_retention_days(MAX_ACCESS_LOG_RETENTION_DAYS + 1).is_err());
    }
}
