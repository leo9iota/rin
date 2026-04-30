//! Core database operations and storage logic for the embedded indexer.
//!
//! This module abstracts over the SurrealDB engine (`SurrealKv` for persistence, 
//! `Mem` for testing), enforcing strict schemas and deterministic idempotency 
//! when storing EVM event logs locally.

use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use surrealdb::engine::local::{Mem, SurrealKv};

/// Provides a thread-safe wrapper around the embedded SurrealDB engine.
///
/// This struct holds an active database connection and exposes strictly typed
/// methods for bulk ingestion and structured retrieval. It manages the `rin` 
/// namespace and `events` database natively.
pub struct Database {
    /// The underlying SurrealDB connection operating on a local KV engine.
    db: Surreal<Db>,
}

impl Database {
    /// Initializes the default persistent SurrealDB instance on disk.
    ///
    /// The database is bound to a local file (`rin_data.db`) and enforces
    /// schema initialization upon creation.
    ///
    /// # Errors
    /// Returns `Err` if the process lacks file system permissions to create/lock
    /// the `.db` file, or if the initial schema definitions fail to execute.
    pub async fn new() -> anyhow::Result<Self> {
        // NOTE: SurrealKv acquires an exclusive file lock. The CLI and API cannot
        // run concurrently against the exact same `.db` file in the same OS process space.
        let db = Surreal::new::<SurrealKv>("rin_data.db").await?;
        Self::setup_schema(&db).await?;
        Ok(Self { db })
    }

    /// Initializes a pure in-memory SurrealDB instance for isolated testing.
    ///
    /// This bypasses all persistent IO operations, ensuring clean execution
    /// without side-effects.
    ///
    /// // NOTE: This function relies on the `kv-mem` feature flag in `Cargo.toml`.
    /// // It guarantees that `cargo test` runs do not create permanent `.db` files 
    /// // on disk, preventing file-lock contention and database corruption when multiple 
    /// // integration tests run concurrently.
    ///
    /// # Errors
    /// Returns `Err` if the in-memory engine fails to initialize or if schema
    /// execution fails.
    pub async fn new_in_memory() -> anyhow::Result<Self> {
        let db = Surreal::new::<Mem>(()).await?;
        Self::setup_schema(&db).await?;
        Ok(Self { db })
    }

    /// Executes the canonical setup queries to enforce data integrity.
    ///
    /// # Errors
    /// Returns `Err` if the syntax is invalid or SurrealDB rejects the `SCHEMAFULL`
    /// constraints.
    async fn setup_schema(db: &Surreal<Db>) -> anyhow::Result<()> {
        db.use_ns("rin").use_db("events").await?;
        
        // Define strict typed schema boundaries and create analytical indices.
        // NOTE: Utilizing `SCHEMAFULL` ensures that any future malformed dynamic 
        // payloads are rejected eagerly at the database layer rather than failing silently.
        let setup_query = r#"
            DEFINE TABLE event SCHEMAFULL;
            DEFINE FIELD block_number ON event TYPE int;
            DEFINE FIELD tx_hash ON event TYPE string;
            DEFINE FIELD address ON event TYPE string;
            DEFINE FIELD log_index ON event TYPE int;
            DEFINE FIELD payload ON event TYPE object;
            
            DEFINE INDEX idx_block_number ON TABLE event COLUMNS block_number;
            DEFINE INDEX idx_address ON TABLE event COLUMNS address;
        "#;
        db.query(setup_query).await?;
        Ok(())
    }

    /// Inserts a batch of decoded events into SurrealDB safely.
    ///
    /// # Errors
    /// Returns `Err` if the underlying batch `upsert` fails or violates constraints.
    pub async fn insert_event_batch(&self, events: Vec<serde_json::Value>) -> anyhow::Result<()> {
        // Chunking prevents overwhelming the embedded KV engine with massive allocations.
        for chunk in events.chunks(1000) {
            // Bulk upsert into the "event" table to guarantee idempotency.
            // NOTE: The `{tx_hash}_{log_index}` unique identifier ensures that if the indexer
            // crashes and restarts from an earlier block, duplicate events will cleanly
            // overwrite their previous iterations rather than causing primary key collisions.
            let _created: Vec<serde_json::Value> = self.db.upsert("event").content(chunk.to_vec()).await?;
        }
        Ok(())
    }

    /// Fetches a paginated sequence of decoded events from SurrealDB.
    ///
    /// # Errors
    /// Returns `Err` if the SurrealQL parsing fails or if the bound limits are invalid.
    pub async fn fetch_events(&self, limit: u32) -> anyhow::Result<Vec<serde_json::Value>> {
        let mut result = self.db.query("SELECT * FROM event LIMIT $limit")
            .bind(("limit", limit))
            .await?;
            
        let events: Vec<serde_json::Value> = result.take(0)?;
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_insert_empty_batch() {
        let db = Database::new_in_memory().await.unwrap();
        let result = db.insert_event_batch(vec![]).await;
        assert!(result.is_ok());
        
        let events = db.fetch_events(10).await.unwrap();
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn test_insert_event_batch_chunking() {
        let db = Database::new_in_memory().await.unwrap();
        
        let mut events = Vec::new();
        for i in 0..1500 {
            events.push(json!({
                "id": format!("0xhash_{}", i),
                "block_number": 100,
                "tx_hash": "0xhash",
                "address": "0xabc",
                "log_index": i,
                "payload": {}
            }));
        }
        
        let result = db.insert_event_batch(events).await;
        assert!(result.is_ok());
        
        // Verify all 1500 records were stored
        let mut res = db.db.query("SELECT count() FROM event GROUP ALL").await.unwrap();
        let count_obj: Option<serde_json::Value> = res.take(0).unwrap();
        
        if let Some(obj) = count_obj {
            let count = obj.get("count").unwrap().as_i64().unwrap();
            assert_eq!(count, 1500);
        } else {
            panic!("Expected count object");
        }
    }
}
