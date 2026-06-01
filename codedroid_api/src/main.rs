use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

mod diagnostics;
mod error_suggestions;
mod git;
mod handlers;
mod lsp;
mod models;
mod runner;
mod terminal;
mod utils;

use handlers::{
    add_package, copy_file, create_dir, delete_file, format_code, get_completions, get_definition,
    get_hover, get_references, move_file, pick_directory, read_file, run_code, run_command,
    scan_project, stop_process, sync_file,
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/run", post(run_code))
        .route("/run_command", post(run_command))
        .route("/stop", post(stop_process))
        .route("/add_package", post(add_package))
        .route("/sync_file", post(sync_file))
        .route("/delete_file", post(delete_file))
        .route("/copy_file", post(copy_file))
        .route("/move_file", post(move_file))
        .route("/create_dir", post(create_dir))
        .route("/complete", post(get_completions))
        .route("/definition", post(get_definition))
        .route("/references", post(get_references))
        .route("/format", post(format_code))
        .route("/read_file", post(read_file))
        .route("/hover", post(get_hover))
        .route("/scan_project", post(scan_project))
        .route("/pick_directory", post(pick_directory))
        .route("/diagnostics", post(diagnostics::get_diagnostics_handler))
        .route(
            "/error_suggestions",
            post(error_suggestions::get_error_suggestions_handler),
        )
        .route("/ping", get(|| async { "pong" }))
        .nest("/terminal", terminal::router())
        .nest("/git", git::router())
        .layer(CorsLayer::permissive());

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("🚀 Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
