use axum::{Router, routing::get, routing::post, Extension};
use std::net::SocketAddr;
use std::sync::Arc;
use async_graphql::{Schema, EmptyMutation, EmptySubscription};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::response::{Html, IntoResponse};

use rin_core::db::Database;

mod graphql;
use graphql::schema::{AppSchema, QueryRoot};

async fn graphql_handler(schema: Extension<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting Rin API server...");

    // Initialize the database (must not run concurrently with CLI)
    let db = Arc::new(Database::new().await?);

    // Build the schema
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(db.clone())
        .finish();

    let app = Router::new()
        .route("/", get(graphql_playground))
        .route("/graphql", post(graphql_handler))
        .layer(Extension(schema));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);
    println!("GraphQL Playground: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
