# tspawn

[![crates.io](https://img.shields.io/crates/v/tspawn.svg)](https://crates.io/crates/tspawn)
[![docs.rs](https://img.shields.io/docsrs/tspawn)](https://docs.rs/tspawn)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/tspawn.svg)](#license)

`tspawn` is a Rust library that provides a thread-safe wrapper around `Arc<RwLock<T>>` with convenient cloning semantics and powerful async task spawning macros. It simplifies working with shared mutable state in concurrent and asynchronous Rust applications.

## Features

- **Thread-safe shared state**: Built on `parking_lot::RwLock` for better performance than `std::sync::RwLock`
- **Convenient cloning**: Clone the wrapper without explicit `Arc::clone()` calls
- **Async task macros**: Powerful `tspawn!` macro for spawning tokio tasks with automatic cloning
- **Multiple access patterns**: Support for read-only, write-only, and mixed access patterns
- **No poisoning**: Uses `parking_lot` which doesn't have lock poisoning
- **Zero-cost abstractions**: Minimal overhead over manual `Arc<RwLock<T>>` usage

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tspawn = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use tspawn::{A, tspawn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a shared counter
    let counter = A::new(0);
    
    // Spawn a task that increments the counter
    tspawn!(mut counter, {
        *counter += 1;
        println!("Counter incremented to: {}", *counter);
    }).await?;
    
    // Read the final value
    println!("Final counter value: {}", counter.get());
    
    Ok(())
}
```

## Core Types

### `A<T>` - The Main Wrapper

`A<T>` is a thread-safe wrapper around `Arc<RwLock<T>>` that provides convenient methods for accessing and modifying shared data:

```rust
use tspawn::A;

let data = A::new(42);

// Read access
let value = data.get(); // Returns a clone of the inner value
let guard = data.read(); // Returns a read guard

// Write access
data.set(100); // Set a new value
data.update(|x| *x += 1); // Update using a closure
let mut guard = data.write(); // Returns a write guard
```

## The `tspawn!` Macro

The `tspawn!` macro simplifies spawning tokio tasks with shared state. It automatically clones the necessary data and provides different access patterns:

### Single Variable Access

```rust
use tspawn::{A, tspawn};

let data = A::new(vec![1, 2, 3]);

// Read-only access
tspawn!(ref data, {
    println!("Data length: {}", data.len());
}).await?;

// Write access
tspawn!(mut data, {
    data.push(4);
    println!("Added element, new length: {}", data.len());
}).await?;

// Clone access (moves the cloned wrapper)
tspawn!(data, {
    let value = data.get();
    println!("Current value: {:?}", value);
}).await?;
```

### Multiple Variable Access

```rust
use tspawn::{A, tspawn};

let x = A::new(10);
let y = A::new(20);

// Multiple read access
tspawn!(ref x, ref y, {
    println!("Sum: {}", *x + *y);
}).await?;

// Mixed access patterns
tspawn!(mut x, ref y, {
    *x += *y;
    println!("x updated to: {}", *x);
}).await?;
```

### Complex Scenarios

```rust
use tspawn::{A, tspawn};

let a = A::new(5);
let b = A::new(10);
let c = A::new(15);

// Three variables with mixed access
tspawn!(mut a, ref b, c, {
    *a = *b + *c.read();
    println!("Updated a: {}", *a);
}).await?;
```

## Advanced Usage

### Working with Complex Types

```rust
use tspawn::A;
use std::collections::HashMap;

#[derive(Clone)]
struct UserData {
    name: String,
    score: i32,
}

let users = A::new(HashMap::<String, UserData>::new());

// Add a user
users.update(|map| {
    map.insert("alice".to_string(), UserData {
        name: "Alice".to_string(),
        score: 100,
    });
});

// Read user data
if let Some(user) = users.read().get("alice") {
    println!("User: {}, Score: {}", user.name, user.score);
}
```

### Integration with async/await

```rust
use tspawn::{A, tspawn};

async fn process_data(data: A<Vec<i32>>) {
    tspawn!(mut data, {
        data.sort();
        data.reverse();
        println!("Processed data: {:?}", *data);
    }).await.unwrap();
}

#[tokio::main]
async fn main() {
    let numbers = A::new(vec![3, 1, 4, 1, 5, 9]);
    process_data(numbers).await;
}
```

## API Reference

### `A<T>` Methods

- `new(value: T) -> Self` - Create a new wrapper
- `get() -> T` - Get a clone of the inner value (requires `T: Clone`)
- `set(value: T)` - Set a new value
- `update<F>(f: F)` - Update the value using a closure
- `read() -> RwLockReadGuard<'_, T>` - Get a read guard
- `write() -> RwLockWriteGuard<'_, T>` - Get a write guard
- `from_inner(Arc<RwLock<T>>) -> Self` - Create from existing Arc<RwLock<T>>
- `into_inner(self) -> Arc<RwLock<T>>` - Convert back to Arc<RwLock<T>>

### `tspawn!` Macro Variants

- `tspawn!(var, { code })` - Clone the wrapper into the task
- `tspawn!(ref var, { code })` - Read access within the task
- `tspawn!(mut var, { code })` - Write access within the task
- `tspawn!(ref var1, ref var2, { code })` - Multiple read access
- `tspawn!(mut var1, ref var2, { code })` - Mixed access patterns
- And more combinations for up to 3 variables

## Performance

`tspawn` uses `parking_lot::RwLock` instead of `std::sync::RwLock` for better performance:

- **Faster lock operations**: `parking_lot` is typically 2-3x faster
- **No poisoning overhead**: Simpler error handling
- **Better fairness**: More predictable scheduling between readers and writers
- **Smaller memory footprint**: More efficient lock implementation

## Examples

See the [`examples/`](examples/) directory for more comprehensive examples:

- [Basic usage](examples/basic.rs)
- [Multiple tasks](examples/multiple_tasks.rs)
- [Complex data structures](examples/complex_types.rs)

## Requirements

- Rust 1.70 or later
- `tokio` runtime for async functionality

## Contributing

We welcome contributions! Please feel free to submit issues or pull requests on [GitHub](https://github.com/modeckrus/tspawn).

### Development

```bash
# Clone the repository
git clone https://github.com/modeckrus/tspawn.git
cd tspawn

# Run tests
cargo test

# Run examples
cargo run --example basic

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy
```

## License

This project is licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

## Acknowledgments

This library is inspired by common patterns in Rust async programming and aims to reduce boilerplate when working with shared mutable state across tokio tasks.
