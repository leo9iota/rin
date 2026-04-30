//! Manages the dynamic decoding of arbitrary EVM event logs into structured payloads.
//!
//! This module acts as the translation layer between raw binary EVM logs 
//! (fetched via RPC) and our generic `serde_json` database schema. By leveraging 
//! Alloy's `dyn_abi`, we can decode events without compiling static Solidity bindings,
//! allowing the indexer to adapt to any contract address and signature purely via configuration.

use alloy::dyn_abi::{DynSolEvent, DynSolValue, Specifier};
use alloy::rpc::types::Log;
use anyhow::{Context, Result};
use serde_json::{Map, Value};

/// A highly dynamic EVM event decoder that leverages Alloy's
/// `dyn_abi` to parse arbitrary hexadecimal log data into typed JSON.
///
/// This struct holds a pre-resolved `DynSolEvent` signature, ensuring
/// that the expensive string-parsing logic is only executed once during initialization
/// rather than on every incoming log.
#[derive(Debug)]
pub struct DynamicDecoder {
    /// The resolved ABI event signature used to decode matching logs.
    event: DynSolEvent,
}

impl DynamicDecoder {
    /// Attempts to parse a raw Solidity event signature string into a functional decoder.
    ///
    /// # Errors
    /// Returns `Err` if the provided signature is malformed, lacks required types,
    /// or cannot be fully resolved by the internal `alloy::json_abi::Event` parser.
    pub fn new(signature: &str) -> Result<Self> {
        // NOTE: We parse through `json_abi::Event` first because `DynSolEvent` lacks
        // a direct `FromStr` implementation for complex human-readable signatures.
        let ev = alloy::json_abi::Event::parse(signature)
            .map_err(|e| anyhow::anyhow!("Failed to parse signature string: {}", e))?;
            
        let event = ev.resolve()
            .map_err(|e| anyhow::anyhow!("Failed to resolve event signature: {}", e))?;
            
        Ok(Self { event })
    }

    /// Transforms a raw EVM `Log` into a standardized `serde_json::Value` payload.
    ///
    /// This method maps all `indexed` and unindexed (`body`) data fields into
    /// a generic object structure, while wrapping it with deterministic identifiers
    /// required for idempotent database storage.
    ///
    /// # Errors
    /// Returns `Err` if the log's topics or data slice do not match the expected
    /// structure of the configured event signature.
    pub fn decode_log(&self, log: &Log) -> Result<Value> {
        // In alloy 0.1+, `log.topics()` returns a slice of B256, which `decode_log_parts` consumes.
        let topics = log.topics().iter().copied();
        let data = &log.data().data;
        
        let decoded = self.event.decode_log_parts(topics, data)
            .context("Failed to decode log parts")?;
            
        let mut payload = Map::new();
        
        // Map indexed parameters
        for (i, val) in decoded.indexed.into_iter().enumerate() {
            payload.insert(format!("indexed_{i}"), dyn_to_json(val));
        }
        
        // Map unindexed (body) parameters
        for (i, val) in decoded.body.into_iter().enumerate() {
            payload.insert(format!("data_{i}"), dyn_to_json(val));
        }
        
        let mut root = Map::new();
        
        // NOTE: We extract these fields eagerly and provide safe defaults if missing,
        // as some RPC providers may return pending logs without block metadata.
        // E.g., `transaction_hash` or `log_index` can legitimately be absent on unmined blocks.
        let tx_hash = log.transaction_hash.map(|h| h.to_string()).unwrap_or_default();
        let log_idx = log.log_index.unwrap_or_default();
        let block_num = log.block_number.unwrap_or_default();
        let address = log.inner.address.to_string();
        
        // Deterministic ID generation (`{tx_hash}_{log_index}`) guarantees that 
        // repeated fetches of the same block range do not duplicate records in SurrealDB.
        root.insert("id".to_string(), Value::String(format!("{}_{}", tx_hash, log_idx)));
        root.insert("block_number".to_string(), Value::Number(serde_json::Number::from(block_num)));
        root.insert("tx_hash".to_string(), Value::String(tx_hash));
        root.insert("address".to_string(), Value::String(address));
        root.insert("log_index".to_string(), Value::Number(serde_json::Number::from(log_idx)));
        root.insert("payload".to_string(), Value::Object(payload));
        
        Ok(Value::Object(root))
    }
}

/// Helper function to safely map Alloy's `DynSolValue` into standard JSON scalar types.
///
/// // TODO: For a full production implementation, this needs to recursively handle
/// // array and tuple types, as well as applying checksums to address hex strings.
fn dyn_to_json(val: DynSolValue) -> Value {
    Value::String(format!("{val:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, B256, Bytes, U256, address, b256, bytes};
    use alloy::rpc::types::Log as RpcLog;
    use std::str::FromStr;

    #[test]
    fn test_decode_invalid_signature() {
        // Given an invalid signature
        let invalid_sig = "Transfer(address, address,,)";
        
        // When we attempt to create a decoder
        let result = DynamicDecoder::new(invalid_sig);
        
        // Then it should fail gracefully
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse signature string"));
    }

    #[test]
    fn test_decode_erc20_transfer() {
        // Given an ERC20 Transfer signature
        let sig = "Transfer(address indexed from, address indexed to, uint256 value)";
        let decoder = DynamicDecoder::new(sig).unwrap();

        // And a mocked raw log for a 100 token transfer
        let t0 = b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"); // Keccak256("Transfer(address,address,uint256)")
        let from_topic = b256!("0000000000000000000000001111111111111111111111111111111111111111");
        let to_topic = b256!("0000000000000000000000002222222222222222222222222222222222222222");
        
        let mut log = RpcLog::default();
        log.inner.address = address!("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
        log.transaction_hash = Some(b256!("0x1234567812345678123456781234567812345678123456781234567812345678"));
        log.log_index = Some(42);
        log.block_number = Some(100);
        
        // In alloy 0.1, the inner `data` type for Log is `LogData`. 
        // We can create a mocked Log by setting inner log data.
        let log_data = alloy::primitives::LogData::new_unchecked(
            vec![t0, from_topic, to_topic],
            Bytes::from(U256::from(100).to_be_bytes_vec()),
        );
        log.inner.data = log_data;

        // When we decode the log
        let result = decoder.decode_log(&log).expect("Failed to decode log");

        // Then we get a correctly structured JSON object
        let obj = result.as_object().unwrap();
        assert_eq!(obj["block_number"], 100);
        assert_eq!(obj["log_index"], 42);
        assert_eq!(obj["address"], "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
        assert_eq!(obj["id"], "0x1234567812345678123456781234567812345678123456781234567812345678_42");
        
        let payload = obj["payload"].as_object().unwrap();
        // The address formatting in alloy may output lowercase hex
        assert!(payload["indexed_0"].as_str().unwrap().contains("1111111111111111111111111111111111111111"));
        assert!(payload["indexed_1"].as_str().unwrap().contains("2222222222222222222222222222222222222222"));
        // The value is 100
        assert_eq!(payload["data_0"], "100");
    }
}
