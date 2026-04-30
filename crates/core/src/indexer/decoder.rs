use alloy::dyn_abi::DynSolEvent;
use alloy::rpc::types::Log;
use anyhow::{Context, Result};
use serde_json::Value;

pub struct DynamicDecoder {
    event: DynSolEvent,
}

impl DynamicDecoder {
    pub fn new(signature: &str) -> Result<Self> {
        // Alloy dyn-abi allows parsing Solidity signatures directly.
        let event: DynSolEvent = signature
            .parse()
            .context("Failed to parse event signature")?;
        Ok(Self { event })
    }

    pub fn decode_log(&self, log: &Log) -> Result<Value> {
        // Parse the log's topics and data based on the event signature
        // In alloy 0.1+, `log.topics()` returns a slice of B256
        let topics = log.topics();
        // `log.data.data` contains the unindexed data bytes
        let data = &log.data.data;
        
        let decoded = self.event.decode_log_parts(topics, data, true)
            .context("Failed to decode log parts")?;
            
        // Map decoded values to serde_json::Value
        let mut map = serde_json::Map::new();
        for (i, value) in decoded.body.into_iter().enumerate() {
            // DynSolValue can be formatted or serialized
            let key = format!("field_{}", i);
            map.insert(key, serde_json::json!(value.to_string()));
        }
        
        Ok(Value::Object(map))
    }
}
