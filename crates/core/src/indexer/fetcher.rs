use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::Filter;
use alloy::rpc::types::Log;
use anyhow::{Context, Result};
use std::str::FromStr;
use tokio::task::JoinSet;

use crate::pipeline::config::ConfigPayload;

/// Default number of blocks per chunk when splitting a large range
/// into parallel `eth_getLogs` queries. Most public RPC nodes enforce
/// a 10,000 block cap per request, so 5,000 gives comfortable headroom.
const DEFAULT_CHUNK_SIZE: u64 = 5_000;

/// Connects to an EVM RPC node and fetches historical event logs
/// based on the user's configuration payload.
pub struct LogFetcher {
    config: ConfigPayload,
}

impl LogFetcher {
    pub fn new(config: ConfigPayload) -> Self {
        Self { config }
    }

    /// Establish an Alloy HTTP provider, validate the connection by
    /// fetching the chain ID, and return all matching logs for the
    /// configured contract address and event signature.
    ///
    /// # Errors
    /// Returns `Err` if the RPC provider connection fails, the requested block range
    /// is invalid (e.g., `start_block` exceeds `latest_block`), or if the `eth_getLogs`
    /// RPC call fails to execute successfully across the batch chunks.
    ///
    /// # Panics
    /// This function will panic if the underlying tokio runtime fails to join
    /// the concurrent chunk-fetching workers.
    pub async fn fetch_logs(&self) -> Result<Vec<Log>> {
        // Phase 1: Provider initialization
        let rpc_url = self
            .config
            .rpc_url
            .parse()
            .context("Failed to parse RPC URL")?;
        let provider = ProviderBuilder::new().connect_http(rpc_url);

        // Validate connection by requesting the chain ID
        let chain_id = provider
            .get_chain_id()
            .await
            .context("Failed to connect to RPC node: could not fetch chain ID")?;
        tracing::info!(chain_id, "Connected to EVM node");

        // Resolve the latest block number as the upper bound
        let latest_block = provider
            .get_block_number()
            .await
            .context("Failed to fetch latest block number")?;
        let end_block = latest_block;

        let start = self.config.start_block;
        if start > end_block {
            anyhow::bail!(
                "start_block ({start}) exceeds the chain head ({end_block})"
            );
        }

        tracing::info!(start, end_block, "Fetching logs across block range");

        // Phase 2: Chunk the block range
        // NOTE: We proactively slice the total block range into smaller chunks
        // to bypass the hard 10,000 block query limit enforced by providers like Alchemy and Infura.
        let chunks = chunk_block_range(start, end_block, DEFAULT_CHUNK_SIZE);
        tracing::info!(chunk_count = chunks.len(), "Block range split into chunks");

        // Phase 3: Parallel eth_getLogs queries
        let contract_address = Address::from_str(&self.config.contract_address)
            .context("Invalid contract address")?;

        let mut join_set = JoinSet::new();
        for (chunk_start, chunk_end) in chunks {
            let provider = provider.clone();
            let event_sig = self.config.event_signature.clone();
            let addr = contract_address;

            join_set.spawn(async move {
                let filter = Filter::new()
                    .address(addr)
                    .event(&event_sig)
                    .from_block(chunk_start)
                    .to_block(chunk_end);

                let logs = provider.get_logs(&filter).await.with_context(|| {
                    format!("eth_getLogs failed for blocks {chunk_start}..{chunk_end}")
                })?;

                tracing::debug!(
                    chunk_start,
                    chunk_end,
                    log_count = logs.len(),
                    "Chunk fetched"
                );

                Ok::<Vec<Log>, anyhow::Error>(logs)
            });
        }

        // Phase 4: Aggregate results in chronological order
        let mut all_logs: Vec<Log> = Vec::new();
        while let Some(result) = join_set.join_next().await {
            let logs = result.context("Worker task panicked")??;
            all_logs.extend(logs);
        }

        // Sort by block number, then log index within each block
        all_logs.sort_by(|a, b| {
            let block_cmp = a.block_number.cmp(&b.block_number);
            if block_cmp.is_eq() {
                a.log_index.cmp(&b.log_index)
            } else {
                block_cmp
            }
        });

        tracing::info!(total_logs = all_logs.len(), "Log fetching complete");
        Ok(all_logs)
    }
}

/// Divide a `[start, end]` block range into non-overlapping chunks of
/// at most `chunk_size` blocks each. The final chunk may be smaller
/// than `chunk_size` to capture the remaining tail.
fn chunk_block_range(start: u64, end: u64, chunk_size: u64) -> Vec<(u64, u64)> {
    let mut chunks = Vec::new();
    let mut cursor = start;
    while cursor <= end {
        let chunk_end = (cursor + chunk_size - 1).min(end);
        chunks.push((cursor, chunk_end));
        cursor = chunk_end + 1;
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_exact_multiple() {
        let chunks = chunk_block_range(0, 9999, 5000);
        assert_eq!(chunks, vec![(0, 4999), (5000, 9999)]);
    }

    #[test]
    fn chunk_with_remainder() {
        let chunks = chunk_block_range(0, 12000, 5000);
        assert_eq!(chunks, vec![(0, 4999), (5000, 9999), (10000, 12000)]);
    }

    #[test]
    fn chunk_smaller_than_size() {
        let chunks = chunk_block_range(100, 200, 5000);
        assert_eq!(chunks, vec![(100, 200)]);
    }

    #[test]
    fn chunk_single_block() {
        let chunks = chunk_block_range(42, 42, 5000);
        assert_eq!(chunks, vec![(42, 42)]);
    }
}
