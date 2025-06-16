//! # tspawn
//!
//! A thread-safe wrapper around `Arc<RwLock<T>>` with convenient cloning semantics and powerful async task spawning macros.
//!
//! This crate provides a simple but powerful abstraction for sharing mutable state across async tasks
//! in Rust applications. It's built on top of `parking_lot::RwLock` for better performance and
//! includes convenient macros for spawning tokio tasks with automatic cloning.
//!
//! ## Quick Start
//!
//! ```rust
//! use tspawn::{A, tspawn};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a shared counter
//! let counter = A::new(0);
//!
//! // Spawn a task that increments the counter
//! tspawn!(mut counter, {
//!     *counter += 1;
//!     println!("Counter incremented to: {}", *counter);
//! }).await?;
//!
//! // Read the final value
//! println!("Final counter value: {}", counter.get());
//! # Ok(())
//! # }
//! ```
//!
//! ## Core Types
//!
//! - [`A<T>`] - The main thread-safe wrapper around `Arc<RwLock<T>>`
//! - [`tspawn!`] - Macro for spawning tokio tasks with automatic cloning and lock management
//!
//! ## Features
//!
//! - **Thread-safe shared state**: Built on `parking_lot::RwLock` for better performance
//! - **Convenient cloning**: Clone the wrapper without explicit `Arc::clone()` calls
//! - **Async task macros**: Powerful `tspawn!` macro for spawning tokio tasks
//! - **Multiple access patterns**: Support for read-only, write-only, and mixed access
//! - **No poisoning**: Uses `parking_lot` which doesn't have lock poisoning
//!
//! ## Usage Patterns
//!
//! ### Basic Shared State
//!
//! ```rust
//! use tspawn::A;
//!
//! let data = A::new(42);
//!
//! // Read access
//! let value = data.get(); // Returns a clone of the inner value
//! let guard = data.read(); // Returns a read guard
//!
//! // Write access
//! data.set(100); // Set a new value
//! data.update(|x| *x += 1); // Update using a closure
//! ```
//!
//! ### Async Task Spawning
//!
//! ```rust
//! use tspawn::{A, tspawn};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let data = A::new(vec![1, 2, 3]);
//!
//! // Read-only access
//! tspawn!(ref data, {
//!     println!("Data length: {}", data.len());
//! }).await?;
//!
//! // Write access
//! tspawn!(mut data, {
//!     data.push(4);
//!     println!("Added element, new length: {}", data.len());
//! }).await?;
//! # Ok(())
//! # }
//! ```

mod a;
pub use a::A;

// Macro to automatically clone variables and spawn a tokio task
/// Spawns a tokio task with automatic cloning and lock management for shared state.
///
/// The `tspawn!` macro simplifies the common pattern of cloning shared data and spawning
/// async tasks. It now uses `Send`-safe guards from parking_lot, allowing the use of
/// `.await` within the task blocks.
///
/// # Access Patterns
///
/// - `var` - Clones the wrapper into the task (no automatic locking)
/// - `ref var` - Provides read-only access (automatically acquires read lock)
/// - `mut var` - Provides write access (automatically acquires write lock)
///
/// # Expansion Pattern
///
/// The macro expands to the following pattern:
/// ```rust,ignore
/// {
///     let var = ::core::clone::Clone::clone(&var);
///     tokio::spawn({
///         let mut var = var.write(); // or let var = var.read(); for ref
///         async move {
///             // user code here
///         }
///     })
/// }
/// ```
///
/// # Examples
///
/// ## Single Variable
///
/// ```rust
/// use tspawn::{A, tspawn};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let data = A::new(42);
///
/// // Read-only access (no await in block)
/// tspawn!(ref data, {
///     println!("Value: {}", *data);
/// }).await?;
///
/// // Write access (no await in block)
/// tspawn!(mut data, {
///     *data += 1;
///     println!("Updated value: {}", *data);
/// }).await?;
///
/// // Clone access (can use await)
/// tspawn!(data, {
///     let value = data.get();
///     println!("Cloned value: {}", value);
///     // tokio::time::sleep(...).await; // This works here
/// }).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## For Async Operations
///
/// When you need to use `.await` within the task, use the clone pattern:
///
/// ```rust
/// use tspawn::{A, tspawn};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let data = A::new(42);
///
/// tspawn!(data, {
///     tokio::time::sleep(std::time::Duration::from_millis(10)).await;
///     let guard = data.read();
///     println!("Value after delay: {}", *guard);
///     drop(guard); // Explicitly drop guard before next await if needed
/// }).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Returns
///
/// Returns a `JoinHandle<()>` from `tokio::spawn`, which can be awaited to
/// ensure the task completes.
///
/// # Macro Expansion Examples
///
/// The `tspawn!` macro expands to clean, efficient code. Here are some expansion examples:
///
/// ## Single Variable with `ref`
/// ```rust,ignore
/// tspawn!(ref data, { println!("Value: {}", *data); })
/// ```
/// Expands to:
/// ```rust,ignore
/// {
///     let data = ::core::clone::Clone::clone(&data);
///     tokio::spawn({
///         let data = data.read();
///         async move { println!("Value: {}", *data); }
///     })
/// }
/// ```
///
/// ## Multiple Variables with Mixed Modifiers
/// ```rust,ignore
/// tspawn!(a, mut b, ref c, { *b += *c + a.get(); })
/// ```
/// Expands to:
/// ```rust,ignore
/// {
///     let a = ::core::clone::Clone::clone(&a);
///     let b = ::core::clone::Clone::clone(&b);
///     let c = ::core::clone::Clone::clone(&c);
///     tokio::spawn({
///         let mut b = b.write();
///         let c = c.read();
///         async move { *b += *c + a.get(); }
///     })
/// }
/// ```
///
/// The macro is fully variadic and can handle any number of variables with any
/// combination of `ref`, `mut`, and bare modifiers.
#[macro_export]
macro_rules! tspawn {
    // Entry point: parse all variables and body
    ($($input:tt)*) => {
        $crate::tspawn_internal!(@parse [] [] $($input)*)
    };
}

// Internal helper macro for parsing variables and building the task
#[doc(hidden)]
#[macro_export]
macro_rules! tspawn_internal {
    // Base case: no more input, spawn the task
    (@parse [$($clone:tt)*] [$($lock:tt)*] $body:block) => {{
        $($clone)*
        tokio::spawn({
            $($lock)*
            async move $body
        })
    }};

    // Parse: ref var
    (@parse [$($clone:tt)*] [$($lock:tt)*] ref $var:ident, $($rest:tt)*) => {
        $crate::tspawn_internal!(
            @parse
            [$($clone)* let $var = ::core::clone::Clone::clone(&$var);]
            [$($lock)* let $var = $var.read();]
            $($rest)*
        )
    };

    // Parse: mut var
    (@parse [$($clone:tt)*] [$($lock:tt)*] mut $var:ident, $($rest:tt)*) => {
        $crate::tspawn_internal!(
            @parse
            [$($clone)* let $var = ::core::clone::Clone::clone(&$var);]
            [$($lock)* let mut $var = $var.write();]
            $($rest)*
        )
    };

    // Parse: bare var
    (@parse [$($clone:tt)*] [$($lock:tt)*] $var:ident, $($rest:tt)*) => {
        $crate::tspawn_internal!(
            @parse
            [$($clone)* let $var = ::core::clone::Clone::clone(&$var);]
            [$($lock)*]
            $($rest)*
        )
    };

    // Parse: ref var (last variable, no comma)
    (@parse [$($clone:tt)*] [$($lock:tt)*] ref $var:ident $body:block) => {
        $crate::tspawn_internal!(
            @parse
            [$($clone)* let $var = ::core::clone::Clone::clone(&$var);]
            [$($lock)* let $var = $var.read();]
            $body
        )
    };

    // Parse: mut var (last variable, no comma)
    (@parse [$($clone:tt)*] [$($lock:tt)*] mut $var:ident $body:block) => {
        $crate::tspawn_internal!(
            @parse
            [$($clone)* let $var = ::core::clone::Clone::clone(&$var);]
            [$($lock)* let mut $var = $var.write();]
            $body
        )
    };

    // Parse: bare var (last variable, no comma)
    (@parse [$($clone:tt)*] [$($lock:tt)*] $var:ident $body:block) => {
        $crate::tspawn_internal!(
            @parse
            [$($clone)* let $var = ::core::clone::Clone::clone(&$var);]
            [$($lock)*]
            $body
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let data = A::new(42);

        // Test get
        assert_eq!(data.get(), 42);

        // Test set
        data.set(100);
        assert_eq!(data.get(), 100);

        // Test update
        data.update(|x| *x += 1);
        assert_eq!(data.get(), 101);

        // Test geto
        assert_eq!(data.geto(), Some(101));
    }

    #[test]
    fn test_cloning() {
        let original = A::new(vec![1, 2, 3]);
        let cloned = original.clone();

        // Both should see the same data
        assert_eq!(original.get(), vec![1, 2, 3]);
        assert_eq!(cloned.get(), vec![1, 2, 3]);

        // Modify through original
        original.update(|v| v.push(4));

        // Clone should see the change
        assert_eq!(cloned.get(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_guards() {
        let data = A::new(String::from("Hello"));

        // Test read guard
        {
            let guard = data.read();
            assert_eq!(&*guard, "Hello");
        }

        // Test write guard
        {
            let mut guard = data.write();
            guard.push_str(", World!");
        }

        assert_eq!(data.get(), "Hello, World!");
    }

    #[test]
    fn test_from_and_into_inner() {
        use parking_lot::RwLock;
        use std::sync::Arc;

        // Test from_inner
        let arc_lock = Arc::new(RwLock::new(42));
        let data = A::from_inner(arc_lock);
        assert_eq!(data.get(), 42);

        // Test into_inner
        let arc_lock = data.into_inner();
        assert_eq!(*arc_lock.read(), 42);
    }

    #[tokio::test]
    async fn test_tspawn_ref() {
        let data = A::new(42);

        tspawn!(ref data, {
            assert_eq!(*data, 42);
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_tspawn_mut() {
        let data = A::new(42);

        tspawn!(mut data, {
            *data += 10;
        })
        .await
        .unwrap();

        assert_eq!(data.get(), 52);
    }

    #[tokio::test]
    async fn test_tspawn_clone() {
        let data = A::new(42);

        tspawn!(data, {
            let value = data.get();
            assert_eq!(value, 42);
            data.set(100);
        })
        .await
        .unwrap();

        assert_eq!(data.get(), 100);
    }

    #[tokio::test]
    async fn test_tspawn_multiple_ref() {
        let x = A::new(10);
        let y = A::new(20);

        tspawn!(ref x, ref y, {
            assert_eq!(*x + *y, 30);
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_tspawn_mixed_access() {
        let x = A::new(10);
        let y = A::new(20);

        tspawn!(mut x, ref y, {
            *x += *y;
        })
        .await
        .unwrap();

        assert_eq!(x.get(), 30);
        assert_eq!(y.get(), 20);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let data = A::new(0);
        let mut handles = vec![];

        // Spawn multiple tasks that increment the counter
        for _ in 0..10 {
            let data_clone = data.clone();
            handles.push(tokio::spawn(async move {
                data_clone.update(|x| *x += 1);
            }));
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // All increments should have been applied
        assert_eq!(data.get(), 10);
    }

    #[tokio::test]
    async fn test_tspawn_variadic_patterns() {
        // Test 1 variable
        let a = A::new(1);
        tspawn!(a, {
            assert_eq!(a.get(), 1);
        })
        .await
        .unwrap();

        // Test 1 variable with ref
        tspawn!(ref a, {
            assert_eq!(*a, 1);
        })
        .await
        .unwrap();

        // Test 1 variable with mut
        tspawn!(mut a, {
            *a += 1;
        })
        .await
        .unwrap();
        assert_eq!(a.get(), 2);

        // Test 3 variables with mixed modifiers
        let b = A::new(10);
        let c = A::new(20);
        tspawn!(ref a, mut b, c, {
            *b += *a;
            let c_val = c.get();
            assert_eq!(*b, 12);
            assert_eq!(c_val, 20);
        })
        .await
        .unwrap();
        assert_eq!(b.get(), 12);

        // Test 5 variables with all different patterns
        let d = A::new(30);
        let e = A::new(40);
        tspawn!(a, ref b, mut c, d, ref e, {
            *c += *b + *e;
            assert_eq!(*c, 72); // 20 + 12 + 40
            assert_eq!(a.get(), 2);
            assert_eq!(d.get(), 30);
        })
        .await
        .unwrap();
        assert_eq!(c.get(), 72);
    }

    #[tokio::test]
    async fn test_tspawn_many_variables() {
        // Test with 8 variables to exceed the old hardcoded limit
        let v0 = A::new(0);
        let v1 = A::new(1);
        let v2 = A::new(2);
        let v3 = A::new(3);
        let v4 = A::new(4);
        let v5 = A::new(5);
        let v6 = A::new(6);
        let v7 = A::new(7);

        tspawn!(v0, mut v1, ref v2, v3, ref v4, mut v5, v6, ref v7, {
            *v1 += *v2 + *v4 + *v7; // 1 + 2 + 4 + 7 = 14
            *v5 += v0.get() + v3.get() + v6.get(); // 5 + 0 + 3 + 6 = 14
        })
        .await
        .unwrap();

        assert_eq!(v1.get(), 14);
        assert_eq!(v5.get(), 14);
    }
}
