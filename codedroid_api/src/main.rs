#[tokio::main]
async fn main() {
    if let Err(e) = codedroid_api::start_server().await {
        eprintln!("🚀 Server failed to start: {}", e);
    }
}
