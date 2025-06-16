//! Basic usage example of the tspawn crate.
//!
//! This example demonstrates the fundamental operations of the `A<T>` wrapper
//! and the `tspawn!` macro for spawning async tasks.

use tspawn::{tspawn, A};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic tspawn Example ===\n");

    // Create a shared counter
    let counter = A::new(0);
    println!("Initial counter value: {}", counter.get());

    // Basic operations
    counter.set(5);
    println!("After set(5): {}", counter.get());

    counter.update(|x| *x += 10);
    println!("After update(|x| *x += 10): {}", counter.get());

    // Demonstrate cloning
    let counter_clone = counter.clone();
    counter_clone.update(|x| *x *= 2);
    println!("After clone and multiply by 2: {}", counter.get());

    // Async task with read access
    println!("\n--- Async Task Examples ---");

    tspawn!(ref counter, {
        println!("Task 1 - Read access: {}", *counter);
    })
    .await?;

    // Async task with write access
    tspawn!(mut counter, {
        *counter += 100;
        println!("Task 2 - Write access, added 100: {}", *counter);
    })
    .await?;

    // Async task with clone access
    tspawn!(counter, {
        let value = counter.get();
        println!("Task 3 - Clone access: {}", value);

        // We can still modify since we have the cloned wrapper
        counter.update(|x| *x += 1);
        println!("Task 3 - After increment: {}", counter.get());
    })
    .await?;

    println!("\nFinal counter value: {}", counter.get());

    Ok(())
}
