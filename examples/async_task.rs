use tspawn::*;

async fn some_async_fn() -> i32 {
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    println!("Async function completed");
    10
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Async Task Example ===\n");

    // Create a shared counter
    let counter = A::new(0);
    println!("Initial counter value: {}", counter.get());
    tspawn!(mut counter, {
        *counter += some_async_fn().await;
    })
    .await?;
    println!("Counter after async task: {}", counter.get());
    Ok(())
}
