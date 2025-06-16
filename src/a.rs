//! Thread-safe wrapper around `Arc<RwLock<T>>` with convenient cloning and access methods.

use parking_lot::{ArcRwLockWriteGuard, RawRwLock, RwLock};
use std::sync::Arc;

/// A thread-safe wrapper around `Arc<RwLock<T>>` that provides convenient cloning semantics
/// and easy access to the inner value.
///
/// `A<T>` is designed to simplify working with shared mutable state in concurrent and
/// asynchronous Rust applications. It automatically handles the `Arc` cloning and provides
/// convenient methods for reading and writing the inner value.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use tspawn::A;
///
/// let data = A::new(42);
///
/// // Read the value
/// assert_eq!(data.get(), 42);
///
/// // Update the value
/// data.set(100);
/// assert_eq!(data.get(), 100);
///
/// // Update using a closure
/// data.update(|x| *x += 1);
/// assert_eq!(data.get(), 101);
/// ```
///
/// ## Cloning for Multiple References
///
/// ```rust
/// use tspawn::A;
///
/// let original = A::new(vec![1, 2, 3]);
/// let cloned = original.clone();
///
/// // Both references point to the same data
/// original.update(|v| v.push(4));
/// assert_eq!(cloned.get(), vec![1, 2, 3, 4]);
/// ```
///
/// ## Working with Guards
///
/// ```rust
/// use tspawn::A;
///
/// let data = A::new(String::from("Hello"));
///
/// // Read guard
/// {
///     let guard = data.read();
///     assert_eq!(&*guard, "Hello");
/// }
///
/// // Write guard
/// {
///     let mut guard = data.write();
///     guard.push_str(", World!");
/// }
///
/// assert_eq!(data.get(), "Hello, World!");
/// ```
pub struct A<T> {
    value: Arc<RwLock<T>>,
}

impl<T> Clone for A<T> {
    /// Creates a new reference to the same shared data.
    ///
    /// This is a cheap operation that only clones the `Arc`, not the inner data.
    /// All cloned instances will share the same underlying data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tspawn::A;
    ///
    /// let original = A::new(42);
    /// let cloned = original.clone();
    ///
    /// original.set(100);
    /// assert_eq!(cloned.get(), 100); // Both see the same data
    /// ```
    fn clone(&self) -> Self {
        A {
            value: Arc::clone(&self.value),
        }
    }
}

impl<T> A<T> {
    /// Creates a new `A<T>` wrapping the given value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tspawn::A;
    ///
    /// let data = A::new(42);
    /// assert_eq!(data.get(), 42);
    /// ```
    pub fn new(value: T) -> Self {
        A {
            value: Arc::new(RwLock::new(value)),
        }
    }

    /// Returns a clone of the inner value.
    ///
    /// This method requires that `T` implements `Clone`. It acquires a read lock,
    /// clones the inner value, and returns it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tspawn::A;
    ///
    /// let data = A::new(String::from("Hello"));
    /// let value = data.get();
    /// assert_eq!(value, "Hello");
    /// ```
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.value.read().clone()
    }

    /// Returns `Some(T)` containing a clone of the inner value.
    ///
    /// This is a convenience method that converts the inner value to an `Option`.
    /// It's equivalent to `Some(self.get())`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tspawn::A;
    ///
    /// let data = A::new(42);
    /// assert_eq!(data.geto(), Some(42));
    /// ```
    pub fn geto(&self) -> Option<T>
    where
        T: Clone,
    {
        self.value.read().clone().into()
    }

    /// Sets the inner value to the provided value.
    ///
    /// This method acquires a write lock and replaces the current value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tspawn::A;
    ///
    /// let data = A::new(42);
    /// data.set(100);
    /// assert_eq!(data.get(), 100);
    /// ```
    pub fn set(&self, value: T) {
        *self.value.write() = value;
    }

    /// Updates the inner value using a closure.
    ///
    /// This method acquires a write lock and calls the provided closure with
    /// a mutable reference to the inner value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tspawn::A;
    ///
    /// let data = A::new(vec![1, 2, 3]);
    /// data.update(|v| v.push(4));
    /// assert_eq!(data.get(), vec![1, 2, 3, 4]);
    /// ```
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut guard = self.value.write();
        f(&mut guard);
    }

    /// Returns a read guard for the inner value.
    ///
    /// This allows for more complex read operations without cloning the data.
    /// The guard will automatically release the lock when dropped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tspawn::A;
    ///
    /// let data = A::new(String::from("Hello"));
    /// let guard = data.read();
    /// assert_eq!(&*guard, "Hello");
    /// // Lock is automatically released when guard is dropped
    /// ```
    pub fn read(&self) -> parking_lot::ArcRwLockReadGuard<RawRwLock, T> {
        self.value.read_arc()
    }

    /// Returns a write guard for the inner value.
    ///
    /// This allows for more complex write operations. The guard will
    /// automatically release the lock when dropped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tspawn::A;
    ///
    /// let data = A::new(String::from("Hello"));
    /// {
    ///     let mut guard = data.write();
    ///     guard.push_str(", World!");
    /// } // Lock is released here
    /// assert_eq!(data.get(), "Hello, World!");
    /// ```
    pub fn write(&self) -> ArcRwLockWriteGuard<RawRwLock, T> {
        self.value.write_arc()
    }

    /// Creates an `A<T>` from an existing `Arc<RwLock<T>>`.
    ///
    /// This is useful when you already have an `Arc<RwLock<T>>` and want to
    /// wrap it in the `A<T>` interface.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tspawn::A;
    /// use parking_lot::RwLock;
    /// use std::sync::Arc;
    ///
    /// let arc_lock = Arc::new(RwLock::new(42));
    /// let data = A::from_inner(arc_lock);
    /// assert_eq!(data.get(), 42);
    /// ```
    pub fn from_inner(value: Arc<RwLock<T>>) -> Self {
        A { value }
    }

    /// Consumes the `A<T>` and returns the inner `Arc<RwLock<T>>`.
    ///
    /// This is useful when you need to work with the underlying `Arc<RwLock<T>>`
    /// directly or interface with APIs that expect this type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tspawn::A;
    ///
    /// let data = A::new(42);
    /// let arc_lock = data.into_inner();
    /// assert_eq!(*arc_lock.read(), 42);
    /// ```
    pub fn into_inner(self) -> Arc<RwLock<T>> {
        self.value
    }
}
