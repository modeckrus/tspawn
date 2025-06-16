//! Multiple tasks example showing concurrent access patterns.
//!
//! This example demonstrates how multiple async tasks can work with
//! shared state using different access patterns.

use std::time::Duration;
use tspawn::{tspawn, A};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multiple Tasks Example ===\n");

    let shared_data = A::new(vec![1, 2, 3]);
    let counter = A::new(0);

    println!("Initial data: {:?}", shared_data.get());
    println!("Initial counter: {}\n", counter.get());

    // Spawn multiple tasks that will run concurrently
    let mut handles = vec![];

    // Task 1: Read the shared data (no await - guards are not Send)
    handles.push(tspawn!(ref shared_data, ref counter, {
        println!(
            "Task 1 - Data length: {}, Counter: {}",
            shared_data.len(),
            *counter
        );
    }));

    // Task 2: Modify the shared data (no await - guards are not Send)
    handles.push(tspawn!(mut shared_data, {
        shared_data.push(4);
        println!("Task 2 - Added element 4, new data: {:?}", *shared_data);
    }));

    // Task 3: Increment counter (no await - guards are not Send)
    handles.push(tspawn!(mut counter, {
        *counter += 10;
        println!("Task 3 - Incremented counter to: {}", *counter);
    }));

    // Task 4: Read and modify both
    handles.push(tspawn!(ref shared_data, mut counter, {
        tokio::time::sleep(Duration::from_millis(40)).await;
        *counter += shared_data.len() as i32;
        println!(
            "Task 4 - Counter updated based on data length: {}",
            *counter
        );
    }));

    // Task 5: Complex operation with cloned access
    let shared_data_clone = shared_data.clone();
    let counter_clone = counter.clone();
    handles.push(tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(60)).await;
        let data = shared_data_clone.get();

        println!("Task 5 - Processing data: {:?}", data);

        // Simulate some processing
        let sum: i32 = data.iter().sum();
        counter_clone.update(|c| *c += sum);

        println!(
            "Task 5 - Added sum ({}) to counter, new value: {}",
            sum,
            counter_clone.get()
        );
    }));

    // Wait for all tasks to complete
    for handle in handles {
        handle.await?;
    }

    println!("\n--- Final Results ---");
    println!("Final data: {:?}", shared_data.get());
    println!("Final counter: {}", counter.get());

    // Demonstrate that the data is still accessible and consistent
    shared_data.update(|data| data.push(5));
    println!("After adding 5: {:?}", shared_data.get());

    Ok(())
}
