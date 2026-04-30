use async_graphql::{Context, Object, Schema, EmptyMutation, EmptySubscription};
use rin_core::db::Database;
use std::sync::Arc;

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn events(&self, ctx: &Context<'_>, limit: Option<u32>) -> async_graphql::Result<Vec<async_graphql::Value>> {
        let db = ctx.data::<Arc<Database>>()?;
        let limit = limit.unwrap_or(50);
        
        let events = db.fetch_events(limit).await?;
        
        // Convert serde_json::Value to async_graphql::Value
        let mut gql_events = Vec::with_capacity(events.len());
        for ev in events {
            let json_bytes = serde_json::to_vec(&ev)?;
            let gql_val: async_graphql::Value = serde_json::from_slice(&json_bytes)?;
            gql_events.push(gql_val);
        }
        
        Ok(gql_events)
    }
}
