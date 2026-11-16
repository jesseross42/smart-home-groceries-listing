//! One poll of the household's Google Tasks list.
//!
//! For each open task: record it as an intake request, **then** delete it
//! from Google. Never the other way round — a task is only deleted once its
//! request is safely stored, so a crash or a database error can never lose
//! one. A task whose delete failed is seen again next poll, recognised by its
//! id, and its delete is retried; no second card appears.
//!
//! A task with subtasks is a heading, not an item ([`outline`]): it is
//! deleted only after every subtask under it was stored and deleted.

use sqlx::PgPool;

use super::api::RemoteTask;
use super::outline::{note_of, outline, subtasks_of};
use super::registry::GoogleTasks;
use super::title::parse_title;
use super::{link_repository, log, sign_in};
use crate::error::ApiError;
use crate::models::db::{GoogleTasksLink, IntakeSource};
use crate::models::schemas::PollReport;
use crate::services::encryption::Encryptor;
use crate::services::triage::TriageQueue;
use crate::services::voice::{Delivery, PolledItem, VoiceService};
use crate::services::ws_hub::WsHub;

/// Shown on the settings page when a delete failed.
const NOT_DELETED: &str = "Some tasks could not be deleted from Google; trying again next time";

/// Polls Google Tasks into the confirmation queue.
pub struct TasksPoller<'a> {
    pool: &'a PgPool,
    hub: &'a WsHub,
    triage: &'a TriageQueue,
    google: &'a GoogleTasks,
    encryptor: Option<&'a Encryptor>,
}

impl<'a> TasksPoller<'a> {
    /// Borrows what one poll needs.
    pub fn new(
        pool: &'a PgPool,
        hub: &'a WsHub,
        triage: &'a TriageQueue,
        google: &'a GoogleTasks,
        encryptor: Option<&'a Encryptor>,
    ) -> Self {
        Self {
            pool,
            hub,
            triage,
            google,
            encryptor,
        }
    }

    /// Polls once. `None` when polling is not set up (off, not connected, or
    /// no list chosen) — nothing was asked of Google.
    ///
    /// The outcome is stored on the link (`last_polled_at`, and
    /// `last_poll_error` in words for the settings page).
    pub async fn poll_once(&self) -> Result<Option<PollReport>, ApiError> {
        let _one_at_a_time = self.google.poll_lock.lock().await;
        let Some(link) = link_repository::get(self.pool).await? else {
            return Ok(None);
        };
        let Some((_, list_id)) = link.pollable() else {
            return Ok(None);
        };
        let list_id = list_id.to_string();

        let outcome = self.poll_list(&link, &list_id).await;
        let problem = match &outcome {
            Ok(report) if report.not_deleted > 0 => Some(NOT_DELETED.to_string()),
            Ok(_) => None,
            Err(err) => Some(describe(err)),
        };
        link_repository::record_poll(self.pool, problem.as_deref()).await?;
        match &outcome {
            Ok(report) => log::polled(report),
            Err(err) => log::poll_failed(err),
        }
        outcome.map(Some)
    }

    /// Reads the list and handles every task on it.
    async fn poll_list(
        &self,
        link: &GoogleTasksLink,
        list_id: &str,
    ) -> Result<PollReport, ApiError> {
        let token = sign_in::open(self.encryptor, link)?;
        let tasks = self.google.api.open_tasks(&token, list_id).await?;
        let split = outline(&tasks);
        let mut report = PollReport::default();
        let mut gone = Vec::new();
        for task in split.items {
            if self.take_task(&token, list_id, task, &mut report).await? {
                gone.push(task.id.as_str());
            }
        }
        for heading in split.headings {
            report.headings += 1;
            let subtasks = subtasks_of(heading, &tasks);
            if subtasks.iter().all(|id| gone.contains(id)) {
                self.delete(&token, list_id, &heading.id, &mut report).await;
            }
        }
        Ok(report)
    }

    /// Records one task, then deletes it from Google. True when it is gone
    /// from Google.
    async fn take_task(
        &self,
        token: &str,
        list_id: &str,
        task: &RemoteTask,
        report: &mut PollReport,
    ) -> Result<bool, ApiError> {
        let Some(parsed) = parse_title(&task.title) else {
            report.blank += 1;
            return Ok(false);
        };
        let item = PolledItem {
            external_id: task.id.clone(),
            raw_text: parsed.raw_text,
            name: parsed.name,
            quantity: parsed.quantity,
            note: note_of(task),
        };
        let (_, delivery) = VoiceService::new(self.pool, self.hub)
            .record_polled(IntakeSource::Tasks, &item, self.triage)
            .await?;
        match delivery {
            Delivery::Recorded => report.recorded += 1,
            Delivery::AlreadySeen => report.already_seen += 1,
        }
        // Stored; only now may the task leave Google.
        Ok(self.delete(token, list_id, &task.id, report).await)
    }

    /// Deletes one task from Google. True when it is gone; a failure is
    /// counted and tried again next poll.
    async fn delete(
        &self,
        token: &str,
        list_id: &str,
        task_id: &str,
        report: &mut PollReport,
    ) -> bool {
        match self.google.api.delete_task(token, list_id, task_id).await {
            Ok(()) => true,
            Err(err) => {
                log::delete_failed(task_id, &err);
                report.not_deleted += 1;
                false
            }
        }
    }
}

/// A failure in words a person can act on. A server-side fault is not
/// described: its cause (a database error, say) stays in the log.
fn describe(err: &ApiError) -> String {
    match err {
        ApiError::Internal(_) => "Something went wrong on the home server — see its log".into(),
        ApiError::Misconfigured(message) => format!("Setting missing: {message}"),
        other => other.to_string(),
    }
}
