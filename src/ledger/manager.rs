use diesel::{r2d2, Insertable, PgConnection, Queryable, Selectable};
use diesel::r2d2::ConnectionManager;
use serde::{Deserialize, Serialize};
use crate::schema::ledger as LedgerTable;
use diesel::prelude::*;
use anyhow::{Result, anyhow};
use uuid::Uuid;
use chrono::NaiveDateTime;
use bigdecimal::BigDecimal;

#[derive(Deserialize, Serialize, diesel_derive_enum::DbEnum, Debug)]
#[ExistingTypePath = "crate::schema::sql_types::LedgerEntryType"]
#[serde(rename_all = "kebab-case")]
pub enum LedgerEntryType {
    #[db_rename = "on-chain"]
    OnChain,
    #[db_rename = "off-chain"] 
    OffChain,
}

#[derive(Deserialize, Serialize, diesel_derive_enum::DbEnum, Debug)]
#[ExistingTypePath = "crate::schema::sql_types::TransactionType"]
pub enum TransactionType {
    Deposit,
    Withdrawal,
}

#[derive(Deserialize, Serialize, Queryable, Selectable, Debug)]
#[diesel(table_name = LedgerTable)]
pub struct LedgerEntry {
    pub id: Uuid,
    pub address: String,
    pub entry_type: Option<LedgerEntryType>,
    pub on_chain_transaction_version: Option<BigDecimal>,
    pub off_chain_transaction_hash: Option<String>,
    pub transaction_type: Option<TransactionType>,
    pub payment_method_id: Option<Uuid>,
    pub timestamp: Option<NaiveDateTime>,
}

#[derive(Deserialize, Serialize, Insertable)]
#[diesel(table_name = LedgerTable)]
pub struct CreateLedgerEntry {
    pub address: String,
    pub entry_type: Option<LedgerEntryType>,
    pub on_chain_transaction_version: Option<BigDecimal>,
    pub off_chain_transaction_hash: Option<String>,
    pub transaction_type: Option<TransactionType>,
    pub payment_method_id: Option<Uuid>,
    pub timestamp: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct LedgerManager {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>
}

impl LedgerManager {
    pub fn new(pool: r2d2::Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }

    pub async fn create_entry(&mut self, req: CreateLedgerEntry) -> Result<Uuid> {
        use crate::schema::ledger::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };

        let inserted_id = diesel::insert_into(LedgerTable::table)
            .values(&req)
            .returning(id)
            .get_result::<Uuid>(&mut conn)?;

        Ok(inserted_id)
    }

    pub async fn get_entry_by_id(&mut self, entry_id: Uuid) -> Result<Option<LedgerEntry>> {
        use crate::schema::ledger::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };

        let result = ledger
            .filter(id.eq(entry_id))
            .select(LedgerEntry::as_select())
            .first::<LedgerEntry>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub async fn get_entries_by_address(&mut self, addr: String) -> Result<Vec<LedgerEntry>> {
        use crate::schema::ledger::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };

        let entries = ledger
            .filter(address.eq(addr))
            .select(LedgerEntry::as_select())
            .load::<LedgerEntry>(&mut conn)?;

        Ok(entries)
    }

    pub async fn get_entries_by_payment_method(&mut self, method_id: Uuid) -> Result<Vec<LedgerEntry>> {
        use crate::schema::ledger::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };

        let entries = ledger
            .filter(payment_method_id.eq(method_id))
            .select(LedgerEntry::as_select())
            .load::<LedgerEntry>(&mut conn)?;

        Ok(entries)
    }

    pub async fn get_entries_by_transaction_type(&mut self, tx_type: TransactionType) -> Result<Vec<LedgerEntry>> {
        use crate::schema::ledger::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };

        let entries = ledger
            .filter(transaction_type.eq(tx_type))
            .select(LedgerEntry::as_select())
            .load::<LedgerEntry>(&mut conn)?;

        Ok(entries)
    }

    pub async fn get_all_entries(&mut self) -> Result<Vec<LedgerEntry>> {
        use crate::schema::ledger::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };

        let entries = ledger
            .select(LedgerEntry::as_select())
            .load::<LedgerEntry>(&mut conn)?;

        Ok(entries)
    }

    pub async fn delete_entry(&mut self, entry_id: Uuid) -> Result<bool> {
        use crate::schema::ledger::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };

        let deleted_count = diesel::delete(ledger.filter(id.eq(entry_id)))
            .execute(&mut conn)?;

        Ok(deleted_count > 0)
    }

    pub async fn update_entry(&mut self, entry_id: Uuid, updates: CreateLedgerEntry) -> Result<bool> {
        use crate::schema::ledger::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };

        let updated_count = diesel::update(ledger.filter(id.eq(entry_id)))
            .set((
                address.eq(updates.address),
                entry_type.eq(updates.entry_type),
                on_chain_transaction_version.eq(updates.on_chain_transaction_version),
                off_chain_transaction_hash.eq(updates.off_chain_transaction_hash),
                transaction_type.eq(updates.transaction_type),
                payment_method_id.eq(updates.payment_method_id),
                timestamp.eq(updates.timestamp),
            ))
            .execute(&mut conn)?;

        Ok(updated_count > 0)
    }
}