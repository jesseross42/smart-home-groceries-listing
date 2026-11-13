//! Which requests one page of the access log may hold.

use super::HEALTH_CHECK_PATH;

/// The `/settings/logs` page's filters. A blank user or path means "any".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccessLogFilter {
    hide_health_checks: bool,
    user: Option<String>,
    path_prefix: Option<String>,
}

impl AccessLogFilter {
    /// Builds the filter from what the page asked for, trimming both texts.
    pub fn new(hide_health_checks: bool, user: Option<String>, path: Option<String>) -> Self {
        Self {
            hide_health_checks,
            user: non_blank(user),
            path_prefix: non_blank(path),
        }
    }

    /// The exact path to leave out: the health probe, when it is hidden.
    pub fn skip_path(&self) -> Option<&'static str> {
        self.hide_health_checks.then_some(HEALTH_CHECK_PATH)
    }

    /// Only this user id's requests, when given.
    pub fn user(&self) -> Option<&str> {
        self.user.as_deref()
    }

    /// Only paths starting with this, when given.
    pub fn path_prefix(&self) -> Option<&str> {
        self.path_prefix.as_deref()
    }
}

/// The trimmed text, or `None` when nothing is left.
fn non_blank(text: Option<String>) -> Option<String> {
    text.map(|t| t.trim().to_string()).filter(|t| !t.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_filters_mean_any() {
        let filter = AccessLogFilter::new(false, Some("  ".into()), Some(String::new()));

        assert_eq!(filter.user(), None);
        assert_eq!(filter.path_prefix(), None);
        assert_eq!(filter.skip_path(), None);
    }

    #[test]
    fn filters_are_trimmed() {
        let filter = AccessLogFilter::new(true, Some(" dev-user ".into()), Some(" /api/g".into()));

        assert_eq!(filter.user(), Some("dev-user"));
        assert_eq!(filter.path_prefix(), Some("/api/g"));
        assert_eq!(filter.skip_path(), Some(HEALTH_CHECK_PATH));
    }
}
