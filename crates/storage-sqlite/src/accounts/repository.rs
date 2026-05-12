use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{self, Pool};
use diesel::sqlite::SqliteConnection;
use std::sync::Arc;

use crate::db::{get_connection, WriteHandle};
use crate::errors::StorageError;
use crate::schema::accounts;
use crate::schema::accounts::dsl::*;
use crate::schema::{daily_account_valuation, holdings_snapshots};

use super::model::AccountDB;
use mizan_core::accounts::{Account, AccountRepositoryTrait, AccountUpdate, NewAccount};
use mizan_core::errors::Result;

/// Repository for managing account data in the database
pub struct AccountRepository {
    pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl AccountRepository {
    /// Creates a new AccountRepository instance
    pub fn new(
        pool: Arc<Pool<r2d2::ConnectionManager<SqliteConnection>>>,
        writer: WriteHandle,
    ) -> Self {
        Self { pool, writer }
    }
}

// Implement the trait
#[async_trait]
impl AccountRepositoryTrait for AccountRepository {
    /// Creates a new account
    async fn create(&self, new_account: NewAccount) -> Result<Account> {
        new_account.validate()?;

        self.writer
            .exec_tx(move |tx| {
                let mut account_db: AccountDB = new_account.into();
                account_db.id = uuid::Uuid::new_v4().to_string();

                diesel::insert_into(accounts::table)
                    .values(&account_db)
                    .execute(tx.conn())
                    .map_err(StorageError::from)?;

                let payload_db = account_db.clone();
                let account: Account = account_db.into();
                tx.insert(&payload_db)?;

                Ok(account)
            })
            .await
    }

    async fn update(&self, account_update: AccountUpdate) -> Result<Account> {
        account_update.validate()?;

        // Capture which optional fields were explicitly set before conversion
        let is_archived_provided = account_update.is_archived.is_some();
        let tracking_mode_provided = account_update.tracking_mode.is_some();

        self.writer
            .exec_tx(move |tx| {
                let mut account_db: AccountDB = account_update.into();

                let existing = accounts
                    .select(AccountDB::as_select())
                    .find(&account_db.id)
                    .first::<AccountDB>(tx.conn())
                    .map_err(StorageError::from)?;

                // Preserve fields that shouldn't change
                account_db.currency = existing.currency;
                account_db.created_at = existing.created_at;
                account_db.updated_at = chrono::Utc::now().naive_utc();

                // Preserve broker-managed fields (only set by broker sync, not user form)
                account_db.provider_account_id = existing.provider_account_id;
                account_db.platform_id = existing.platform_id;
                account_db.provider = existing.provider;
                account_db.account_number = existing.account_number;
                account_db.meta = existing.meta;

                // Preserve is_archived and tracking_mode if not explicitly provided
                if !is_archived_provided {
                    account_db.is_archived = existing.is_archived;
                }
                if !tracking_mode_provided {
                    account_db.tracking_mode = existing.tracking_mode;
                }

                diesel::update(accounts.find(&account_db.id))
                    .set(&account_db)
                    .execute(tx.conn())
                    .map_err(StorageError::from)?;

                let payload_db = account_db.clone();
                let account: Account = account_db.into();
                tx.update(&payload_db)?;

                Ok(account)
            })
            .await
    }

    /// Retrieves an account by its ID
    fn get_by_id(&self, account_id: &str) -> Result<Account> {
        let mut conn = get_connection(&self.pool)?;

        let account = accounts
            .select(AccountDB::as_select())
            .find(account_id)
            .first::<AccountDB>(&mut conn)
            .map_err(StorageError::from)?;

        Ok(account.into())
    }

    /// Lists accounts in the database, optionally filtering by active status, archived status, and account IDs
    fn list(
        &self,
        is_active_filter: Option<bool>,
        is_archived_filter: Option<bool>,
        account_ids: Option<&[String]>,
    ) -> Result<Vec<Account>> {
        let mut conn = get_connection(&self.pool)?;

        let mut query = accounts::table.into_boxed();

        if let Some(active) = is_active_filter {
            query = query.filter(is_active.eq(active));
        }

        if let Some(archived) = is_archived_filter {
            query = query.filter(is_archived.eq(archived));
        }

        if let Some(ids) = account_ids {
            query = query.filter(id.eq_any(ids));
        }

        let results = query
            .select(AccountDB::as_select())
            .order((is_active.desc(), is_archived.asc(), name.asc()))
            .load::<AccountDB>(&mut conn)
            .map_err(StorageError::from)?;

        let accounts_list: Vec<Account> = results.into_iter().map(Account::from).collect();
        Ok(accounts_list)
    }

    /// Deletes an account by its ID and returns the number of deleted records.
    ///
    /// Atomically cleans up rows in `holdings_snapshots` and
    /// `daily_account_valuation` that reference this account. Those tables
    /// were created (2025-04-21 migration) without `FOREIGN KEY ... ON
    /// DELETE CASCADE`, so the row-level delete here is the only thing
    /// preventing orphaned snapshots/valuations from sticking around after
    /// an account goes away — orphans that the dashboard then surfaces as
    /// phantom historical value for a deleted account.
    ///
    /// Activities, goal allocations, broker sync state, and import runs
    /// all have real CASCADE constraints (init_db + refactor_asset_model
    /// migrations), so we don't touch them explicitly here.
    async fn delete(&self, account_id_param: &str) -> Result<usize> {
        let id_to_delete_owned = account_id_param.to_string();
        let event_entity_id = id_to_delete_owned.clone();
        self.writer
            .exec_tx(move |tx| {
                // Clean orphan-prone tables first. Order doesn't strictly
                // matter (no FK between them) but doing it pre-account-delete
                // keeps the cleanup atomic with the account row removal.
                diesel::delete(
                    holdings_snapshots::table
                        .filter(holdings_snapshots::account_id.eq(&id_to_delete_owned)),
                )
                .execute(tx.conn())
                .map_err(StorageError::from)?;

                diesel::delete(
                    daily_account_valuation::table
                        .filter(daily_account_valuation::account_id.eq(&id_to_delete_owned)),
                )
                .execute(tx.conn())
                .map_err(StorageError::from)?;

                let affected_rows = diesel::delete(accounts.find(&id_to_delete_owned))
                    .execute(tx.conn())
                    .map_err(StorageError::from)?;

                if affected_rows > 0 {
                    tx.delete::<AccountDB>(event_entity_id.clone());
                }
                Ok(affected_rows)
            })
            .await
    }
}
