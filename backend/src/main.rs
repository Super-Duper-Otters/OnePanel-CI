use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

mod git;
mod handlers;
mod state;

use git::GitStatus;
use handlers::*;
use state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::list_directories,
        handlers::add_directory,
        handlers::remove_directory,
    ),
    components(
        schemas(handlers::CreateDirectoryRequest, handlers::DirectoryResponse, GitStatus)
    ),
    tags(
        (name = "directories", description = "Directory management endpoints")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState::new();

    let app = Router::new()
        .route(
            "/api/directories",
            get(list_directories)
                .post(add_directory)
                .delete(remove_directory),
        )
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
