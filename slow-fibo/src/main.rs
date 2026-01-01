use eyre::Result;
use slow_fibo::init_telemetry;
use tracing::{info, instrument};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize Telemetry
    init_telemetry();

    info!("🚀 System initializing...");

    // 2. Run Async Workflow
    perform_task().await?;

    Ok(())
}

#[instrument]
async fn perform_task() -> Result<()> {
    // The 'instrument' macro automatically creates a span for this function
    info!("Working on task...");

    // Simulate work for illustration ... but this should be replaced by actual core functionality
    // to be tested as part of the build and release process
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    info!("Task complete.");
    Ok(())
}
