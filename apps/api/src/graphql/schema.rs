//! Defines the core GraphQL schema and root resolvers for the API layer.
//!
//! This module exposes the primary `AppSchema` used by the Axum server to serve
//! analytical queries. It bridges the incoming GraphQL requests with the underlying
//! SurrealDB data store.

use async_graphql::{Context, Object, Schema, EmptyMutation, EmptySubscription};
use rin_core::db::Database;
use std::sync::Arc;

/// The canonical GraphQL schema type for the application.
/// Currently configured strictly for Queries (read-only); Mutations and Subscriptions
/// are explicitly empty.
pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

/// The root query object aggregating all top-level analytical GraphQL endpoints.
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Retrieves a paginated sequence of raw EVM events stored in the database.
    ///
    /// # Arguments
    /// * `limit` - An optional cap on the number of events returned. Defaults to 50 if absent.
    ///
    /// # Errors
    /// Returns `Err` if the internal Database extraction fails, the database is offline,
    /// or if the query times out before completion.
    async fn events(&self, ctx: &Context<'_>, limit: Option<u32>) -> async_graphql::Result<Vec<async_graphql::Json<serde_json::Value>>> {
        let db = ctx.data::<Arc<Database>>()?;
        let limit = limit.unwrap_or(50);
        
        let events = db.fetch_events(limit).await?;
        
        // NOTE: We eagerly map the dynamic JSON strings into `async_graphql::Json`
        // so that the generic `serde_json::Value` payload passes through the schema
        // unescaped and strictly structured.
        let mut gql_events = Vec::with_capacity(events.len());
        for ev in events {
            gql_events.push(async_graphql::Json(ev));
        }
        
        Ok(gql_events)
    }
}
