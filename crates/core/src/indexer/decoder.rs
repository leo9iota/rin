use alloy::dyn_abi::DynSolEvent;
use alloy::rpc::types::Log;
use anyhow::{Context, Result};
use serde_json::{json, Value};

pub struct DynamicDecoder {
    event: DynSolEvent,
}

impl DynamicDecoder {
    pub fn new(signature: &str) -> Result<Self> {
        // Alloy dyn-abi allows parsing Solidity signatures directly.
        // TODO: For now we mock the parsing so it compiles. 
        // In a real implementation we would use `alloy_dyn_abi::parser::parse_event_decl` or similar.
        unimplemented!("Need to properly parse signature: {}", signature);
    }

    pub fn decode_log(&self, log: &Log) -> Result<Value> {
        // Parse the log's topics and data based on the event signature
        // In alloy 0.1+, `log.topics()` returns a slice of B256
        let topics = log.topics().iter().copied();
        // `log.data.data` contains the unindexed data bytes
        let data = &log.data().data;
        
        let decoded = self.event.decode_log_parts(topics, data)
            .context("Failed to decode log parts")?;
            
        // Map decoded values to serde_json::Value payload
        let mut payload = serde_json::Map::new();
        for (i, value) in decoded.body.into_iter().enumerate() {
            let key = format!("field_{}", i);
            payload.insert(key, json!(format!("{:?}", value)));
        }
        
        let tx_hash = log.transaction_hash.map(|h| h.to_string()).unwrap_or_default();
        let log_idx = log.log_index.unwrap_or_default();
        // Deterministic ID ensures idempotency if the same block is indexed twice
        let id = format!("{}_{}", tx_hash, log_idx);
        
        let addr = log.address().to_string();
        
        let mut map = serde_json::Map::new();
        map.insert("id".to_string(), json!(id));
        map.insert("block_number".to_string(), json!(log.block_number.unwrap_or_default()));
        map.insert("tx_hash".to_string(), json!(tx_hash));
        map.insert("address".to_string(), json!(addr));
        map.insert("log_index".to_string(), json!(log_idx));
        map.insert("payload".to_string(), Value::Object(payload));
        
        Ok(Value::Object(map))
    }
}
