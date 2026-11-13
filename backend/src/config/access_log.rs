//! Where the access-log file goes (`docs/features/FEATURE_ACCESS_LOGS.md`).
//!
//! Read on its own, before [`Settings`](super::Settings), because logging is
//! set up first so a bad setting can itself be logged.

use std::path::PathBuf;

use super::{optional, parse_or};
use crate::error::ConfigError;

/// Days an `access_logs` row is kept when `ACCESS_LOG_RETENTION_DAYS` is unset.
pub const DEFAULT_ACCESS_LOG_RETENTION_DAYS: u32 = 90;

/// `ACCESS_LOG_DIR`: the folder for the rotating access-log files. Unset
/// means no file — the `access_logs` table is still written.
pub fn access_log_dir() -> Option<PathBuf> {
    optional("ACCESS_LOG_DIR").map(|dir| PathBuf::from(dir.trim()))
}

/// `ACCESS_LOG_RETENTION_DAYS`: days an `access_logs` row is kept before the
/// daily clean-up deletes it. `0` keeps rows forever; unset is
/// [`DEFAULT_ACCESS_LOG_RETENTION_DAYS`]. The files have their own 30-day limit.
pub(super) fn retention_days() -> Result<u32, ConfigError> {
    parse_or(
        "ACCESS_LOG_RETENTION_DAYS",
        DEFAULT_ACCESS_LOG_RETENTION_DAYS,
    )
}
