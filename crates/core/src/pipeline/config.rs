#[derive(Debug, Clone)]
pub struct ConfigPayload {
    pub rpc_url: String,
    pub contract_address: String,
    pub event_signature: String,
    pub start_block: u64,
}
