//! Synchronization primitives for async CANopen operations.
//!
//! Provides traits for async mutex and signal implementations.


/// Async mutex trait with Generic Associated Types.
/// 
/// # Example
/// 
/// ```ignore
/// struct MyMutex;
///
/// impl AsyncMutex for MyMutex {
///     type Mutex<T> = SomeMutex<T>;
///     type Guard<'a, T> = SomeGuard<'a, T> where T: 'a;
///     
///     fn new<T>(value: T) -> Self::Mutex<T> {
///         SomeMutex::new(value)
///     }
///     
///     async fn lock<'a, T>(mutex: &'a Self::Mutex<T>) -> Self::Guard<'a, T> where T: 'a {
///         mutex.lock().await
///     }
/// }
/// ```
pub trait AsyncMutex {
    type Mutex<T>;
    type Guard<'a, T>: core::ops::Deref<Target = T> + core::ops::DerefMut<Target = T>
    where
        Self: 'a,
        T: 'a;
    
    /// Create a new mutex with the given value
    fn new<T>(value: T) -> Self::Mutex<T>;
    
    /// Lock the mutex
    fn lock<'a, T>(mutex: &'a Self::Mutex<T>) -> impl core::future::Future<Output = Self::Guard<'a, T>> where T: 'a, Self: 'a;
}

/// Async signal trait for task communication.
/// 
/// # Example
/// 
/// ```ignore
/// struct MySignal;
/// 
/// impl AsyncSignal for MySignal {
///     type Signal<T: Clone> = SomeSignal<T>;
///     
///     fn new<T: Clone>() -> Self::Signal<T> {
///         SomeSignal::new()
///     }
///     
///     fn reset<T: Clone>(signal: &Self::Signal<T>) {
///         signal.reset();
///     }
///     
///     fn signal<T: Clone>(signal: &Self::Signal<T>, value: T) {
///         signal.signal(value);
///     }
///     
///     async fn wait<T: Clone>(signal: &Self::Signal<T>) -> T {
///         signal.wait().await
///     }
/// }
/// ```
pub trait AsyncSignal {
    type Signal<T: Clone>;
    
    /// Create a new signal
    fn new<T: Clone>() -> Self::Signal<T>;
    
    /// Reset the signal
    fn reset<T: Clone>(signal: &Self::Signal<T>);
    
    /// Signal a value
    fn signal<T: Clone>(signal: &Self::Signal<T>, value: T);
    
    /// Wait for a signal
    fn wait<T: Clone>(signal: &Self::Signal<T>) -> impl core::future::Future<Output = T>;
}

/// Embassy implementations
#[cfg(feature = "embassy")]
pub mod embassy {
    use super::{AsyncMutex, AsyncSignal};
    use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex, signal::Signal};
    
    /// Embassy mutex implementation
    pub struct EmbassyMutex;
    
    impl AsyncMutex for EmbassyMutex {
        type Mutex<T> = Mutex<NoopRawMutex, T>;
        type Guard<'a, T> = embassy_sync::mutex::MutexGuard<'a, NoopRawMutex, T> where T: 'a;
        
        fn new<T>(value: T) -> Self::Mutex<T> {
            Mutex::new(value)
        }
        
        async fn lock<'a, T>(mutex: &'a Self::Mutex<T>) -> Self::Guard<'a, T> where T: 'a {
            mutex.lock().await
        }
    }
    
    /// Embassy signal implementation
    pub struct EmbassySignal;
    
    impl AsyncSignal for EmbassySignal {
        type Signal<T: Clone> = Signal<NoopRawMutex, T>;
        
        fn new<T: Clone>() -> Self::Signal<T> {
            Signal::new()
        }
        
        fn reset<T: Clone>(signal: &Self::Signal<T>) {
            signal.reset();
        }
        
        fn signal<T: Clone>(signal: &Self::Signal<T>, value: T) {
            signal.signal(value);
        }
        
        async fn wait<T: Clone>(signal: &Self::Signal<T>) -> T {
            signal.wait().await
        }
    }
}
