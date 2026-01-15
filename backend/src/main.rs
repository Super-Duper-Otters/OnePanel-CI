#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use axum::{
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Router,
};
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use std::thread;
use tao::event_loop::{ControlFlow, EventLoop};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tray_icon::{
    menu::{Menu, MenuItem},
    TrayIconBuilder,
};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

mod db;
mod docker;
mod fs;
mod git;
mod handlers;
mod icon;
mod mcp_server;
mod models;
mod onepanel;
mod state;

use docker::DockerInfo;
use fs::{FileEntry, ListRequest, ReadFileRequest, ScanRequest};
use git::{CommitInfo, FileStatus, GitStatus};
use handlers::git::{GitLogRequest, GitStatusRequest};
use models::{
    CreateDirectoryRequest, CreateServerRequest, DashboardResponse, DirectoryResponse, OsInfo,
    Repository, Server, ServerResponse,
};
use state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::repository::list_repositories,
        handlers::repository::add_repository,
        handlers::repository::remove_repository,
        handlers::repository::update_docker_config,
        handlers::repository::get_docker_config,
        handlers::fs::list_directory,
        handlers::fs::scan_directory,
        handlers::fs::read_file,
        handlers::git::get_git_log,
        handlers::git::get_git_status,
        handlers::server::list_servers,
        handlers::server::add_server,
        handlers::server::delete_server,
        handlers::server::get_server_status,
        handlers::server::update_server,
        handlers::server::get_server,
        handlers::docker::get_info,
        handlers::docker::list_tags,
        handlers::docker::build_image,
        handlers::docker::list_containers,
        handlers::docker::start_container,
        handlers::docker::stop_container,
        handlers::docker::remove_container,
        handlers::docker::get_container_logs,
        handlers::docker::list_images,
        handlers::docker::pull_image,
        handlers::docker::remove_image,
        handlers::container::list_containers,
        handlers::container::operate_container,
        handlers::container::get_logs,
        handlers::deploy::push_image_to_server,
        handlers::compose::list_composes,
        handlers::compose::get_content,
        handlers::compose::update_content,
        handlers::image::list_images,
        handlers::image::remove_image,
        handlers::compose::operate_compose,
        handlers::image_deployments::get_image_deployments,
    ),
    components(
        schemas(CreateDirectoryRequest, DirectoryResponse, GitStatus, FileEntry, ListRequest, ScanRequest, ReadFileRequest, CommitInfo, FileStatus, GitLogRequest, GitStatusRequest, CreateServerRequest, ServerResponse, DashboardResponse, OsInfo, Server, Repository, DockerInfo, docker::DockerImage, docker::ContainerSummary, docker::PullImageRequest, models::ContainerOperationReq, models::PushImageReq, handlers::compose::GetContentReq, handlers::compose::OperateComposeReq, handlers::image_deployments::ImageDeployment)
    ),
    tags(
        (name = "directories", description = "Directory management endpoints"),
        (name = "fs", description = "File system endpoints"),
        (name = "git", description = "Git operations endpoints"),
        (name = "servers", description = "Server management endpoints"),
        (name = "docker", description = "Docker endpoints"),
        (name = "Container", description = "1Panel Container management")
    )
)]
struct ApiDoc;

#[derive(RustEmbed)]
#[folder = "../frontend/dist"]
struct Assets;

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Assets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains('.') {
                return (StatusCode::NOT_FOUND, "404 Not Found").into_response();
            }
            if let Some(content) = Assets::get("index.html") {
                let mime = mime_guess::from_path("index.html").first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            } else {
                (StatusCode::NOT_FOUND, "404 Not Found").into_response()
            }
        }
    }
}

// Function to run the web server logic
async fn run_server() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,bollard=off".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = db::init_db().await.unwrap();
    let state = AppState::new(db);

    let app = Router::new()
        .route(
            "/api/directories",
            get(handlers::repository::list_repositories)
                .post(handlers::repository::add_repository)
                .delete(handlers::repository::remove_repository),
        )
        .route(
            "/api/directories/config/update",
            axum::routing::post(handlers::repository::update_docker_config),
        )
        .route(
            "/api/directories/config/get",
            axum::routing::post(handlers::repository::get_docker_config),
        )
        .route(
            "/api/fs/list",
            axum::routing::post(handlers::fs::list_directory),
        )
        .route(
            "/api/fs/scan",
            axum::routing::post(handlers::fs::scan_directory),
        )
        .route("/api/fs/read", axum::routing::post(handlers::fs::read_file))
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
                .put(handlers::server::update_server)
                .get(handlers::server::get_server),
        )
        .route(
            "/api/servers/{id}/status",
            get(handlers::server::get_server_status),
        )
        .route(
            "/api/servers/{id}/proxy/{*path}",
            axum::routing::any(handlers::server::proxy_request),
        )
        .route(
            "/api/servers/{id}/docs",
            get(handlers::server::serve_scalar_docs),
        )
        .route("/api/docker/info", get(handlers::docker::get_info))
        .route("/api/docker/tags", get(handlers::docker::list_tags))
        .route(
            "/api/docker/containers",
            get(handlers::docker::list_containers),
        )
        .route(
            "/api/docker/containers/{id}/start",
            axum::routing::post(handlers::docker::start_container),
        )
        .route(
            "/api/docker/containers/{id}/stop",
            axum::routing::post(handlers::docker::stop_container),
        )
        .route(
            "/api/docker/containers/{id}/logs",
            get(handlers::docker::get_container_logs),
        )
        .route(
            "/api/docker/containers/{id}",
            axum::routing::delete(handlers::docker::remove_container),
        )
        .route("/api/docker/images", get(handlers::docker::list_images))
        .route(
            "/api/docker/images/pull",
            axum::routing::post(handlers::docker::pull_image),
        )
        .route(
            "/api/docker/prune",
            axum::routing::post(handlers::docker::prune_images),
        )
        .route(
            "/api/docker/images/{id}",
            axum::routing::delete(handlers::docker::remove_image),
        )
        .route(
            "/api/docker/build",
            axum::routing::post(handlers::docker::build_image),
        )
        .route(
            "/api/servers/{id}/containers",
            get(handlers::container::list_containers),
        )
        .route(
            "/api/servers/{id}/containers/operate",
            axum::routing::post(handlers::container::operate_container),
        )
        .route(
            "/api/servers/{id}/containers/logs",
            get(handlers::container::get_logs),
        )
        .route(
            "/api/servers/{id}/composes",
            get(handlers::compose::list_composes),
        )
        .route(
            "/api/servers/{id}/composes/content",
            axum::routing::post(handlers::compose::get_content),
        )
        .route(
            "/api/servers/{id}/composes/content/update",
            axum::routing::post(handlers::compose::update_content),
        )
        .route(
            "/api/servers/{id}/composes/operate",
            axum::routing::post(handlers::compose::operate_compose),
        )
        .route(
            "/api/servers/{id}/images",
            get(handlers::image::list_images),
        )
        .route(
            "/api/servers/{id}/images/remove",
            axum::routing::post(handlers::image::remove_image),
        )
        .route(
            "/api/deploy/image",
            axum::routing::post(handlers::deploy::push_image_to_server),
        )
        .route(
            "/api/image-deployments",
            get(handlers::image_deployments::get_image_deployments),
        )
        .route("/api/version", get(handlers::version::get_version))
        .route(
            "/api/notifications",
            get(handlers::notifications::list_notifications)
                .post(handlers::notifications::create_notification)
                .delete(handlers::notifications::clear_notifications),
        )
        .route(
            "/sse",
            get(handlers::mcp::sse_handler).post(handlers::mcp::post_handler),
        )
        .route("/mcp", axum::routing::post(handlers::mcp::post_handler))
        .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
        .fallback(static_handler);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Starting OnePanel CI version: {}", env!("APP_VERSION"));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn main() {
    let event_loop = EventLoop::new();

    // Spawn server in a separate thread
    thread::spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(run_server());
    });

    let tray_menu = Menu::new();
    let quit_i = MenuItem::new("Quit", true, None);
    let open_i = MenuItem::new("Open", true, None);
    tray_menu.append_items(&[&open_i, &quit_i]).unwrap();

    let icon = icon::load_icon().unwrap();

    let mut _tray_icon = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("OnePanel CI")
            .with_icon(icon)
            .build()
            .unwrap(),
    );

    let menu_channel = tray_icon::menu::MenuEvent::receiver();
    let tray_channel = tray_icon::TrayIconEvent::receiver();

    // Open browser on startup
    let _ = open::that("http://localhost:3000");

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_i.id() {
                _tray_icon.take();
                *control_flow = ControlFlow::Exit;
            } else if event.id == open_i.id() {
                let _ = open::that("http://localhost:3000");
            }
        }

        if let Ok(event) = tray_channel.try_recv() {
            // On Windows, left click maps to different event than Mac/Linux potentially
            // But tray-icon 0.19 might abstract this.
            // User requested to remove single click open functionality.
            match event {
                // tray_icon::TrayIconEvent::Click { button, .. } => {
                //     if button == tray_icon::MouseButton::Left {
                //         let _ = open::that("http://localhost:3000");
                //     }
                // }
                _ => {}
            }
        }
    });
}
