//! Optional "Start at login": the operating system starts the app when the
//! person signs in (a Launch Agent on macOS, a `Run` registry value on
//! Windows, an autostart entry on Linux).
//!
//! The operating system's own entry is the only record of the choice, so it
//! cannot disagree with a saved setting. A login start carries [`LOGIN_ARG`],
//! and the app then stays in the tray without showing its window.

use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Runtime};
use tauri_plugin_autostart::{AutoLaunchManager, ManagerExt};

/// The argument the operating system passes when it starts the app at login.
pub const LOGIN_ARG: &str = "--started-at-login";

/// The plugin that writes and removes the operating system's login entry.
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_autostart::Builder::new()
        .arg(LOGIN_ARG)
        .build()
}

/// True when `args` (the process arguments) say the app was started at login.
pub fn started_at_login<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter().any(|arg| arg.as_ref() == LOGIN_ARG)
}

/// The operating system's login entry for this app.
pub trait LoginEntry {
    /// Whether the app starts at login now.
    fn is_enabled(&self) -> Result<bool, String>;
    /// Adds or removes the entry.
    fn set_enabled(&self, enabled: bool) -> Result<(), String>;
}

impl LoginEntry for AutoLaunchManager {
    fn is_enabled(&self) -> Result<bool, String> {
        AutoLaunchManager::is_enabled(self).map_err(|err| err.to_string())
    }

    fn set_enabled(&self, enabled: bool) -> Result<(), String> {
        let result = if enabled {
            self.enable()
        } else {
            self.disable()
        };
        result.map_err(|err| err.to_string())
    }
}

/// Flips the entry and answers the new state.
pub fn toggle(entry: &impl LoginEntry) -> Result<bool, String> {
    let enabled = !entry.is_enabled()?;
    entry.set_enabled(enabled)?;
    Ok(enabled)
}

/// Whether the app starts at login. An unreadable entry counts as "no".
pub fn is_enabled<R: Runtime>(app: &AppHandle<R>) -> bool {
    LoginEntry::is_enabled(&*app.autolaunch()).unwrap_or_else(|err| {
        log::warn!("[autostart] cannot read the login entry: {err}");
        false
    })
}

/// Flips "Start at login" for this app.
pub fn toggle_for<R: Runtime>(app: &AppHandle<R>) -> Result<bool, String> {
    let enabled = toggle(&*app.autolaunch())?;
    log::info!("[autostart] start at login is now {enabled}");
    Ok(enabled)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    /// A login entry kept in memory, which can be made to refuse changes.
    struct FakeEntry {
        enabled: Cell<bool>,
        refuses_changes: bool,
    }

    impl FakeEntry {
        fn new(enabled: bool, refuses_changes: bool) -> Self {
            Self {
                enabled: Cell::new(enabled),
                refuses_changes,
            }
        }
    }

    impl LoginEntry for FakeEntry {
        fn is_enabled(&self) -> Result<bool, String> {
            Ok(self.enabled.get())
        }

        fn set_enabled(&self, enabled: bool) -> Result<(), String> {
            if self.refuses_changes {
                return Err("refused".into());
            }
            self.enabled.set(enabled);
            Ok(())
        }
    }

    #[test]
    fn only_the_login_argument_marks_a_login_start() {
        assert!(started_at_login(["grocery-desktop", LOGIN_ARG]));
        assert!(!started_at_login(["grocery-desktop"]));
        assert!(!started_at_login(["grocery-desktop", "--started"]));
        assert!(!started_at_login(Vec::<String>::new()));
    }

    #[test]
    fn toggling_turns_it_on_and_a_second_toggle_undoes_it() {
        let entry = FakeEntry::new(false, false);
        assert_eq!(toggle(&entry), Ok(true));
        assert!(entry.enabled.get());
        assert_eq!(toggle(&entry), Ok(false));
        assert!(!entry.enabled.get());
    }

    #[test]
    fn a_refused_change_leaves_the_entry_as_it_was() {
        let entry = FakeEntry::new(true, true);
        assert_eq!(toggle(&entry), Err("refused".to_string()));
        assert!(entry.enabled.get());
    }
}
