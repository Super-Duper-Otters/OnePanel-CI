use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

mod db;
mod fs;
mod git;
mod handlers;
mod models;
mod onepanel;
mod state;

use fs::{FileEntry, ListRequest, ScanRequest};
use git::{CommitInfo, FileStatus, GitStatus};
use handlers::git::{GitLogRequest, GitStatusRequest};
use handlers::*;
use models::{CreateServerRequest, DashboardResponse, OsInfo, Server, ServerResponse};
use state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::list_directories,
        handlers::add_directory,
        handlers::remove_directory,
        handlers::fs::list_directory,
        handlers::fs::scan_directory,
        handlers::git::get_git_log,
        handlers::git::get_git_status,
        handlers::server::list_servers,
        handlers::server::add_server,
        handlers::server::delete_server,
        handlers::server::get_server_status,
    ),
    components(
        schemas(handlers::CreateDirectoryRequest, handlers::DirectoryResponse, GitStatus, FileEntry, ListRequest, ScanRequest, CommitInfo, FileStatus, GitLogRequest, GitStatusRequest, CreateServerRequest, ServerResponse, DashboardResponse, OsInfo, Server)
    ),
    tags(
        (name = "directories", description = "Directory management endpoints"),
        (name = "fs", description = "File system endpoints"),
        (name = "git", description = "Git operations endpoints"),
        (name = "servers", description = "Server management endpoints")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = db::init_db().await.unwrap();
    let state = AppState::new(db);

    let app = Router::new()
        .route(
            "/api/directories",
            get(list_directories)
                .post(add_directory)
                .delete(remove_directory),
        )
        .route(
            "/api/fs/list",
            axum::routing::post(handlers::fs::list_directory),
        )
        .route(
            "/api/fs/scan",
            axum::routing::post(handlers::fs::scan_directory),
        )
        .route(
            "/api/git/log",
            axum::routing::post(handlers::git::get_git_log),
        )
        .route(
            "/api/git/status",
            axum::routing::post(handlers::git::get_git_status),
        )
        .route(
            "/api/servers",
            get(handlers::server::list_servers).post(handlers::server::add_server),
        )
        .route(
            "/api/servers/{id}",
            axum::routing::delete(handlers::server::delete_server)
                .put(handlers::server::update_server),
        )
        .route(
            "/api/servers/{id}/status",
            get(handlers::server::get_server_status),
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
