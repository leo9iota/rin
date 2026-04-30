use async_graphql::{Context, Object, Schema, EmptyMutation, EmptySubscription};
use rin_core::db::Database;
use std::sync::Arc;

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn events(&self, ctx: &Context<'_>, limit: Option<u32>) -> async_graphql::Result<Vec<async_graphql::Json<serde_json::Value>>> {
        let db = ctx.data::<Arc<Database>>()?;
        let limit = limit.unwrap_or(50);
        
        let events = db.fetch_events(limit).await?;
        
        let mut gql_events = Vec::with_capacity(events.len());
        for ev in events {
            gql_events.push(async_graphql::Json(ev));
        }
        
        Ok(gql_events)
    }
}
