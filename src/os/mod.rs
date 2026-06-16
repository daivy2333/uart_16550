// SPDX-License-Identifier: MIT OR Apache-2.0

//! OS abstraction traits for cross-platform async UART support.
//!
//! This module defines platform-independent traits that allow the async UART
//! driver to be ported to different operating systems by implementing these
//! traits for each target OS.

use core::future::Future;
use core::task::Waker;

/// Task spawning and blocking execution abstraction.
///
/// This trait provides the async runtime primitives needed by the UART driver
/// to spawn background tasks (e.g., copier tasks) and block on futures when
/// synchronous operation is required.
pub trait OsRuntime {
    /// Spawn an async task with a name for debugging.
    ///
    /// The task runs concurrently with the caller. The name is used for
    /// debugging and diagnostics only.
    fn spawn<F>(future: F, name: &str)
    where
        F: Future + Send + 'static,
        F::Output: Send;

    /// Block current thread until future completes.
    ///
    /// This is used in contexts where async/await is not available (e.g.,
    /// during initialization) to synchronously wait for a future to complete.
    fn block_on<F>(future: F) -> F::Output
    where
        F: Future;
}

/// Interrupt handler registration abstraction.
///
/// This trait allows the UART driver to register interrupt handlers for
/// hardware IRQ lines without depending on a specific interrupt controller
/// implementation.
pub trait OsIrq {
    /// Register an interrupt handler for the given IRQ number.
    ///
    /// The handler function receives the IRQ number as its argument. It must
    /// be safe to call from interrupt context.
    fn register_handler(irq_number: usize, handler: fn(usize));
}

/// MMIO memory mapping abstraction.
///
/// This trait provides platform-specific MMIO memory mapping capabilities,
/// allowing the UART driver to access memory-mapped device registers without
/// depending on a specific virtual memory implementation.
pub trait OsMmio {
    /// Map physical MMIO region to virtual memory.
    ///
    /// Returns a non-null pointer to the mapped region.
    ///
    /// # Safety
    ///
    /// - `phys_addr` must be a valid MMIO region for the UART device
    /// - `size` must correctly represent the size of the MMIO region
    /// - The mapped region must remain valid for the lifetime of the UART device
    unsafe fn map_mmio(phys_addr: usize, size: usize) -> core::ptr::NonNull<u8>;

    /// Convert physical address to virtual address.
    ///
    /// This is used when the physical address is already mapped (e.g., via
    /// identity mapping in kernel space) and only a pointer conversion is needed.
    fn phys_to_virt(phys_addr: usize) -> core::ptr::NonNull<u8>;
}

/// IRQ-safe spinlock abstraction using callback pattern.
///
/// This trait provides a spinlock that disables interrupts while the lock is
/// held, preventing deadlocks when the lock is acquired from both process
/// context and interrupt context. The callback pattern ensures that interrupts
/// are always re-enabled when the lock is released.
///
/// The generic parameter `T` is the type of data protected by the lock.
pub trait OsSpinNoIrq<T> {
    /// Create a new spinlock protecting the given value.
    fn new(val: T) -> Self;

    /// Execute a closure with the lock held and IRQs disabled.
    ///
    /// The lock is released and IRQs re-enabled after the closure returns,
    /// regardless of whether the closure panics or returns normally.
    fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R;
}

/// Waker registration and notification abstraction.
///
/// This trait provides a set of wakers that can be notified together, which
/// is used to implement async notification for UART events (e.g., data
/// received, transmit buffer empty). Multiple async tasks can register their
/// wakers to be notified on the same event.
///
/// Implementations must be `Send + Sync` to allow safe sharing across
/// interrupt and task contexts.
pub trait OsWakerSet: Send + Sync {
    /// Create a new empty waker set.
    fn new() -> Self;

    /// Register a waker to be notified on wake events.
    ///
    /// If the waker is already registered, it should be updated (not duplicated).
    fn register(&self, waker: &Waker);

    /// Wake all registered wakers, return count of wakers notified.
    fn wake(&self) -> u32;
}
