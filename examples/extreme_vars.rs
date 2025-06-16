//! Example demonstrating the variadic tspawn! macro with 10 variables.
//! This shows that the macro can handle any number of variables with any modifiers.

use tspawn::{tspawn, A};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Extreme Variables Example (10 variables) ===");

    // Create 10 variables
    let v0 = A::new(0);
    let v1 = A::new(1);
    let v2 = A::new(2);
    let v3 = A::new(3);
    let v4 = A::new(4);
    let v5 = A::new(5);
    let v6 = A::new(6);
    let v7 = A::new(7);
    let v8 = A::new(8);
    let v9 = A::new(9);

    println!("Initial values:");
    println!(
        "v0: {}, v1: {}, v2: {}, v3: {}, v4: {}",
        v0.get(),
        v1.get(),
        v2.get(),
        v3.get(),
        v4.get()
    );
    println!(
        "v5: {}, v6: {}, v7: {}, v8: {}, v9: {}",
        v5.get(),
        v6.get(),
        v7.get(),
        v8.get(),
        v9.get()
    );

    // Use tspawn! with 10 variables with mixed modifiers
    tspawn!(
        v0,     // clone
        mut v1, // write
        ref v2, // read
        v3,     // clone
        ref v4, // read
        mut v5, // write
        v6,     // clone
        ref v7, // read
        mut v8, // write
        ref v9, // read
        {
            // Perform operations using all 10 variables
            *v1 += *v2 + *v4; // v1 = 1 + 2 + 4 = 7
            *v5 += *v7 + *v9; // v5 = 5 + 7 + 9 = 21
            *v8 += v0.get() + v3.get() + v6.get(); // v8 = 8 + 0 + 3 + 6 = 17

            println!("Inside task:");
            println!("  v1 (mut) = {}", *v1);
            println!("  v5 (mut) = {}", *v5);
            println!("  v8 (mut) = {}", *v8);
            println!(
                "  Read-only: v2={}, v4={}, v7={}, v9={}",
                *v2, *v4, *v7, *v9
            );
            println!(
                "  Clones: v0={}, v3={}, v6={}",
                v0.get(),
                v3.get(),
                v6.get()
            );
        }
    )
    .await?;

    println!("\nFinal values:");
    println!(
        "v0: {}, v1: {}, v2: {}, v3: {}, v4: {}",
        v0.get(),
        v1.get(),
        v2.get(),
        v3.get(),
        v4.get()
    );
    println!(
        "v5: {}, v6: {}, v7: {}, v8: {}, v9: {}",
        v5.get(),
        v6.get(),
        v7.get(),
        v8.get(),
        v9.get()
    );

    // Verify the results
    assert_eq!(v1.get(), 7); // 1 + 2 + 4
    assert_eq!(v5.get(), 21); // 5 + 7 + 9
    assert_eq!(v8.get(), 17); // 8 + 0 + 3 + 6

    println!("\n✅ All assertions passed! The variadic macro works with 10 variables.");

    Ok(())
}
