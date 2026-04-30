pub mod fetcher;

pub use fetcher::LogFetcher;

pub struct IndexerEngine;

impl IndexerEngine {
    pub fn new() -> Self {
        Self
    }
}
