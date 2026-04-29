pub mod indexer;
pub mod ui;
pub mod db;

use std::time::Duration;
use tokio::sync::mpsc;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Database
    let _database = db::Database::new().await?;
    println!("Embedded SurrealDB initialized.");

    // 2. Initialize mpsc channel
    let (tx, mut _rx) = mpsc::channel::<String>(100);

    // 3. Initialize Indexer Engine
    let _engine = indexer::IndexerEngine::new();
    println!("Indexer Engine initialized.");

    // 4. Initialize State Machine
    let _app_state = ui::AppState::new();
    println!("UI State Machine initialized.");

    // 5. Dummy async UI harness
    println!("Rin Indexer Starting... (Press Ctrl+C to exit)");
    tx.send("UI Ready".to_string()).await?;

    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("Setup complete. Exiting.");

    Ok(())
}

