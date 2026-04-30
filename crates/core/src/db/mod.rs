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
        Ok(Self { db })
    }

    /// Inserts a batch of decoded events into SurrealDB
    pub async fn insert_event_batch(&self, events: Vec<serde_json::Value>) -> anyhow::Result<()> {
        for chunk in events.chunks(1000) {
            // Bulk insert into the "event" table
            let _created: Vec<serde_json::Value> = self.db.insert("event").content(chunk).await?;
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
