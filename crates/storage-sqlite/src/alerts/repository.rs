//! SQLite implementation of [`mizan_core::alerts::AlertStore`].
//!
//! Writes go through the project's serialised `WriteHandle` to avoid
//! `SQLITE_BUSY` under contention; reads use a connection from the
//! pool directly. The upsert pivots on the unique `fingerprint` so
//! re-running the rule engine is idempotent.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::SqliteConnection;
use std::sync::Arc;
use uuid::Uuid;

use mizan_core::alerts::traits::UpsertOutcome;
use mizan_core::alerts::{AlertStatus, AlertStore, ProposedAlert, SmartAlert};
use mizan_core::Result;

use super::model::SmartAlertDB;
use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::smart_alerts;
use crate::schema::smart_alerts::dsl::*;

pub struct SmartAlertRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl SmartAlertRepository {
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }
}

#[async_trait]
impl AlertStore for SmartAlertRepository {
    async fn upsert(&self, proposal: &ProposedAlert, now: DateTime<Utc>) -> Result<UpsertOutcome> {
        let proposal = proposal.clone();
        let new_id = Uuid::new_v4().to_string();
        let new_row = SmartAlertDB::from_proposal(new_id, &proposal, now);
        let now_rfc = now.to_rfc3339();

        self.writer
            .exec(
                move |conn: &mut SqliteConnection| -> Result<UpsertOutcome> {
                    // Check whether an alert with this fingerprint already
                    // exists. If yes, bump last_seen_at + refresh mutable
                    // fields. If no, insert a fresh row.
                    let existing: Option<SmartAlertDB> = smart_alerts
                        .filter(fingerprint.eq(&new_row.fingerprint))
                        .first::<SmartAlertDB>(conn)
                        .optional()
                        .map_err(StorageError::from)?;

                    match existing {
                        Some(_) => {
                            diesel::update(
                                smart_alerts.filter(fingerprint.eq(&new_row.fingerprint)),
                            )
                            .set((
                                last_seen_at.eq(&now_rfc),
                                severity.eq(&new_row.severity),
                                category.eq(&new_row.category),
                                title.eq(&new_row.title),
                                message.eq(&new_row.message),
                                action_route.eq(&new_row.action_route),
                                metadata_json.eq(&new_row.metadata_json),
                            ))
                            .execute(conn)
                            .map_err(StorageError::from)?;
                            Ok(UpsertOutcome::Updated)
                        }
                        None => {
                            diesel::insert_into(smart_alerts::table)
                                .values(&new_row)
                                .execute(conn)
                                .map_err(StorageError::from)?;
                            Ok(UpsertOutcome::Inserted)
                        }
                    }
                },
            )
            .await
    }

    async fn list(&self, status_filter: Option<AlertStatus>) -> Result<Vec<SmartAlert>> {
        let mut conn = get_connection(&self.pool)?;
        let rows: Vec<SmartAlertDB> = match status_filter {
            Some(s) => smart_alerts
                .filter(status.eq(s.as_str()))
                .order(last_seen_at.desc())
                .load::<SmartAlertDB>(&mut conn)
                .map_err(StorageError::from)?,
            None => smart_alerts
                .order(last_seen_at.desc())
                .load::<SmartAlertDB>(&mut conn)
                .map_err(StorageError::from)?,
        };
        Ok(rows.into_iter().map(SmartAlert::from).collect())
    }

    async fn get(&self, alert_id: &str) -> Result<Option<SmartAlert>> {
        let mut conn = get_connection(&self.pool)?;
        let row = smart_alerts
            .find(alert_id)
            .first::<SmartAlertDB>(&mut conn)
            .optional()
            .map_err(StorageError::from)?;
        Ok(row.map(SmartAlert::from))
    }

    async fn snooze(
        &self,
        alert_id: &str,
        until: DateTime<Utc>,
        _now: DateTime<Utc>,
    ) -> Result<()> {
        let alert_id = alert_id.to_string();
        let until_rfc = until.to_rfc3339();
        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<()> {
                diesel::update(smart_alerts.find(&alert_id))
                    .set((
                        status.eq(AlertStatus::Snoozed.as_str()),
                        snoozed_until.eq(Some(until_rfc)),
                    ))
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await
    }

    async fn dismiss(&self, alert_id: &str, now: DateTime<Utc>) -> Result<()> {
        let alert_id = alert_id.to_string();
        let now_rfc = now.to_rfc3339();
        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<()> {
                diesel::update(smart_alerts.find(&alert_id))
                    .set((
                        status.eq(AlertStatus::Dismissed.as_str()),
                        dismissed_at.eq(Some(now_rfc)),
                    ))
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await
    }

    async fn resolve(&self, alert_id: &str, now: DateTime<Utc>) -> Result<()> {
        let alert_id = alert_id.to_string();
        let now_rfc = now.to_rfc3339();
        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<()> {
                diesel::update(smart_alerts.find(&alert_id))
                    .set((
                        status.eq(AlertStatus::Resolved.as_str()),
                        resolved_at.eq(Some(now_rfc)),
                    ))
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await
    }
}
