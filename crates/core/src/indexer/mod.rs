pub mod fetcher;
pub mod decoder;

pub use fetcher::LogFetcher;
pub use decoder::DynamicDecoder;

pub struct IndexerEngine;

impl IndexerEngine {
    pub fn new() -> Self {
        Self
    }
}
