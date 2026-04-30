use alloy::primitives::{Bytes, U256, address, b256};
use alloy::rpc::types::Log as RpcLog;
use rin_core::db::Database;
use rin_core::indexer::DynamicDecoder;

#[tokio::test]
async fn test_end_to_end_indexing() {
    // 1. Initialize in-memory database
    let db = Database::new_in_memory().await.expect("Failed to init DB");

    // 2. Initialize Decoder for ERC20 Transfer
    let sig = "Transfer(address indexed from, address indexed to, uint256 value)";
    let decoder = DynamicDecoder::new(sig).expect("Failed to init decoder");

    // 3. Mock multiple logs
    let t0 = b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");
    let mut logs = Vec::new();
    
    for i in 0..50 {
        let mut log = RpcLog::default();
        log.inner.address = address!("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
        // Changing tx_hash slightly so they are unique if we want, or keep same
        log.transaction_hash = Some(b256!("0x1234567812345678123456781234567812345678123456781234567812345678"));
        log.log_index = Some(i as u64);
        log.block_number = Some(100 + i as u64);
        
        let from_topic = b256!("0000000000000000000000001111111111111111111111111111111111111111");
        let to_topic = b256!("0000000000000000000000002222222222222222222222222222222222222222");
        
        let log_data = alloy::primitives::LogData::new_unchecked(
            vec![t0, from_topic, to_topic],
            Bytes::from(U256::from(100 * i).to_be_bytes_vec()),
        );
        log.inner.data = log_data;
        logs.push(log);
    }

    // 4. Decode all logs
    let mut decoded_events = Vec::new();
    for log in logs {
        let decoded = decoder.decode_log(&log).expect("Failed to decode");
        decoded_events.push(decoded);
    }

    // 5. Insert into DB
    db.insert_event_batch(decoded_events).await.expect("Failed to insert");

    // 6. Verify fetch
    let fetched = db.fetch_events(100).await.expect("Failed to fetch");
    assert_eq!(fetched.len(), 50);
}
