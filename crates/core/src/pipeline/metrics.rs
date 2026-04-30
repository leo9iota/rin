//! Defines the asynchronous telemetric payload routing across the indexer's concurrent bounds.
//!
//! The MPSC channels utilize these strictly-typed enumerations to broadcast state changes
//! from isolated background worker tasks up to the interactive terminal rendering thread 
//! without introducing lock contention.

/// Represents a discrete state change or progress update from the indexing engine.
///
/// This enum is intended to be passed across standard `tokio::sync::mpsc` channels
/// to safely propagate metrics upwards to UI threads or external telemetry consumers.
#[derive(Debug, Clone)]
pub enum EngineMetrics {
    /// Indicates how many raw EVM logs were successfully retrieved from the RPC endpoint.
    LogsFetched(usize),
    /// Indicates how many raw logs successfully passed through the dynamic ABI decoder.
    LogsDecoded(usize),
    /// Indicates how many structured payloads were committed to the database storage layer.
    EventsInserted(usize),
    /// Emits a stringified representation of a successfully processed event for real-time tailing.
    LogStream(String),
    /// Indicates the overall ingestion pipeline has successfully reached the target chain head.
    PipelineComplete,
    /// Encapsulates a terminal error that caused the pipeline to abort.
    PipelineError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use tokio::task::JoinSet;

    #[tokio::test]
    async fn test_metrics_channel_concurrency() {
        let (tx, mut rx) = mpsc::channel(100);
        
        // Spawn parallel tasks to blast messages
        let mut join_set = JoinSet::new();
        
        for _ in 0..10 {
            let tx_clone = tx.clone();
            join_set.spawn(async move {
                for _ in 0..1000 {
                    tx_clone.send(EngineMetrics::LogsFetched(1)).await.unwrap();
                }
            });
        }
        
        // Drop original sender so receiver terminates when all clones are dropped
        drop(tx);
        
        let mut total_fetched = 0;
        while let Some(msg) = rx.recv().await {
            if let EngineMetrics::LogsFetched(count) = msg {
                total_fetched += count;
            }
        }
        
        while let Some(res) = join_set.join_next().await {
            assert!(res.is_ok());
        }
        
        assert_eq!(total_fetched, 10000);
    }
}
