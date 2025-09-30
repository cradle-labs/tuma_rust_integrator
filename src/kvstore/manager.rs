use diesel::{r2d2, Insertable, PgConnection, Queryable, Selectable};
use diesel::r2d2::ConnectionManager;
use serde::{Deserialize, Serialize};
use crate::schema::kvstore as KVStoreTable;
use diesel::prelude::*;
use anyhow::{Result, anyhow};
use diesel::upsert::excluded;

#[derive(Deserialize, Serialize, Queryable, Selectable, Debug)]
#[diesel(table_name = KVStoreTable)]
pub struct KVPair {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Deserialize, Serialize, Insertable)]
#[diesel(table_name = KVStoreTable)]
pub struct CreateKVPair {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug,Clone)]
pub struct KVStoreManager {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>
}

impl KVStoreManager {
    pub fn new(pool: r2d2::Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }

    pub async fn set(&mut self, key_value: String, value_value: String) -> Result<()> {
        use crate::schema::kvstore::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };

        diesel::insert_into(KVStoreTable::table)
            .values(&CreateKVPair {
                key: key_value.clone(),
                value: Some(value_value.clone()),
            })
            .on_conflict(key)
            .do_update()
            .set(value.eq(value_value))
            .execute(&mut conn)?;

        Ok(())
    }

    pub async fn get(&mut self, key_param: String) -> Result<Option<String>> {
        use crate::schema::kvstore::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };

        let result = kvstore
            .filter(key.eq(key_param))
            .select(KVPair::as_select())
            .first::<KVPair>(&mut conn)
            .optional()?;

        match result {
            Some(kv_pair) => Ok(kv_pair.value),
            None => Ok(None),
        }
    }

    pub async fn delete(&mut self, key_param: String) -> Result<bool> {
        use crate::schema::kvstore::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };

        let deleted_count = diesel::delete(kvstore.filter(key.eq(key_param)))
            .execute(&mut conn)?;

        Ok(deleted_count > 0)
    }

    pub async fn get_all(&mut self) -> Result<Vec<KVPair>> {
        use crate::schema::kvstore::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };

        let pairs = kvstore
            .select(KVPair::as_select())
            .load::<KVPair>(&mut conn)?;

        Ok(pairs)
    }

    pub async fn exists(&mut self, key_param: String) -> Result<bool> {
        use crate::schema::kvstore::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };

        let count = kvstore
            .filter(key.eq(key_param))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(count > 0)
    }

    pub async fn clear(&mut self) -> Result<usize> {
        use crate::schema::kvstore::dsl::*;

        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                println!("Unable to get a db connection {}", e);
                return Err(anyhow!("unable to create a db connection"))
            }
        };

        let deleted_count = diesel::delete(kvstore).execute(&mut conn)?;

        Ok(deleted_count)
    }
}