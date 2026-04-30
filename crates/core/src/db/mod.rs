use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use surrealdb::engine::local::SurrealKv;

pub struct Database {
    db: Surreal<Db>,
}

impl Database {
    pub async fn new() -> anyhow::Result<Self> {
        let db = Surreal::new::<SurrealKv>("rin_data.db").await?;
        db.use_ns("rin").use_db("events").await?;
        
        // Initialize schema
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
        
        Ok(Self { db })
    }

    /// Inserts a batch of decoded events into SurrealDB
    pub async fn insert_event_batch(&self, events: Vec<serde_json::Value>) -> anyhow::Result<()> {
        for chunk in events.chunks(1000) {
            // Bulk upsert into the "event" table to guarantee idempotency
            let _created: Vec<serde_json::Value> = self.db.upsert("event").content(chunk.to_vec()).await?;
        }
        Ok(())
    }

    /// Fetches decoded events from SurrealDB
    pub async fn fetch_events(&self, limit: u32) -> anyhow::Result<Vec<serde_json::Value>> {
        let mut result = self.db.query("SELECT * FROM event LIMIT $limit")
            .bind(("limit", limit))
            .await?;
            
        let events: Vec<serde_json::Value> = result.take(0)?;
        Ok(events)
    }
}
