use surrealdb::Surreal;
use surrealdb::engine::local::Mem;
use surrealdb::engine::local::Db;

pub struct Database {
    db: Surreal<Db>,
}

impl Database {
    pub async fn new() -> anyhow::Result<Self> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("rin").use_db("events").await?;
        Ok(Self { db })
    }
}
