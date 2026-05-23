use axum::{routing::post, Router};
use tower_http::cors::CorsLayer;

mod lsp;
mod models;
mod runner;
mod utils;
mod handlers;

use handlers::{run_code, stop_process, add_package, sync_file, get_completions, delete_file, copy_file, create_dir};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/run", post(run_code))
        .route("/stop", post(stop_process))
        .route("/add_package", post(add_package))
        .route("/sync_file", post(sync_file))
        .route("/delete_file", post(delete_file))
        .route("/copy_file", post(copy_file))
        .route("/create_dir", post(create_dir))
        .route("/complete", post(get_completions))
        .layer(CorsLayer::permissive());

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("🚀 Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
