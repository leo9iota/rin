#[derive(Debug, Clone)]
pub enum EngineMetrics {
    /// Indicates how many logs were fetched so far.
    LogsFetched(usize),
    /// Indicates how many logs have been decoded.
    LogsDecoded(usize),
    /// Indicates how many logs have been inserted into the database.
    EventsInserted(usize),
    /// Indicates the pipeline is finished.
    PipelineComplete,
    /// Indicates an error occurred.
    PipelineError(String),
}
