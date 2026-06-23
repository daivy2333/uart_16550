// SPDX-License-Identifier: MIT OR Apache-2.0

//! Async UART driver with RX/TX copier tasks.
//!
//! Provides [`AsyncUartDriver`] which manages background RX and TX copier
//! tasks with NAPI-style interrupt coalescing for high throughput.
//!
//! The driver is generic over:
//! - `R: OsRuntime` — task spawning abstraction
//! - `W: OsWakerSet` — waker notification abstraction for ring buffers
//! - `U: UartPort` — interior-mutability-safe UART hardware access

use core::fmt;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

#[cfg(feature = "telemetry")]
use core::sync::atomic::Ordering;

use super::isr::{RX_WAKER, TX_WAKER};
use super::ring_buffer::{RingBufRx, RingBufTx};
use crate::os::{OsRuntime, OsWakerSet};

/// NAPI: consecutive successful reads before entering polling mode.
const NAPI_THRESHOLD: u32 = 16;
/// NAPI: batch size in polling mode.
const NAPI_BATCH_SIZE: usize = 64;
/// Copier buffer size for bulk operations.
const COPIER_BUF_SIZE: usize = 1024;

/// UART hardware access abstraction for copier tasks.
///
/// Provides interior-mutability-safe access to UART receive/transmit
/// operations. The OS layer implements this by wrapping `Uart16550` in
/// a suitable lock (e.g., `SpinNoIrq<Uart16550<MmioBackend>>`).
///
/// # Implementor contract
///
/// - `receive_bytes` must read from the UART RBR/THR register
/// - `send_bytes` must write to the UART THR register
/// - Interior mutability must ensure no data races between RX and TX copier
pub trait UartPort: Send + Sync + 'static {
    /// Read available bytes from the UART receive buffer.
    ///
    /// Returns the number of bytes actually read (may be 0 if no data
    /// is available).
    fn receive_bytes(&self, buf: &mut [u8]) -> usize;

    /// Write bytes to the UART transmit buffer.
    ///
    /// Returns the number of bytes actually written (may be 0 if the
    /// transmit buffer is full).
    fn send_bytes(&self, buf: &[u8]) -> usize;
}

/// Async UART driver with RX/TX copier tasks.
///
/// Manages two background tasks:
/// - **RX copier**: reads from UART hardware and pushes to the RX ring buffer
/// - **TX copier**: pops from the TX ring buffer and writes to UART hardware
///
/// The RX copier uses NAPI-style interrupt coalescing: after
/// [`NAPI_THRESHOLD`] consecutive successful reads, it switches to
/// polling mode with [`NAPI_BATCH_SIZE`] batch reads per iteration.
///
/// # Usage
///
/// The driver is created as a `&'static` reference (typically via `static`)
/// and passed to `start_rx_copier` / `start_tx_copier` to spawn the
/// background tasks.
pub struct AsyncUartDriver<R: OsRuntime, W: OsWakerSet, U: UartPort> {
    /// RX ring buffer — data flows from UART to consumers.
    pub rx: RingBufRx<W>,
    /// TX ring buffer — data flows from producers to UART.
    pub tx: RingBufTx<W>,
    uart: &'static U,
    #[cfg(feature = "telemetry")]
    /// Diagnostic counters for TX copier behavior (only available
    /// with the `telemetry` feature).
    pub telemetry: crate::async_::telemetry::Telemetry,
    _runtime: PhantomData<R>,
}

// SAFETY: All fields are Send+Sync:
// - RingBufRx<W>/RingBufTx<W> have explicit unsafe Send+Sync impls
// - &'static U is Send+Sync when U: Send+Sync (guaranteed by UartPort)
// - PhantomData<R> is Send+Sync unconditionally
unsafe impl<R: OsRuntime, W: OsWakerSet, U: UartPort> Send
    for AsyncUartDriver<R, W, U>
{}
// SAFETY: Same reasoning as Send — all fields are Sync-safe.
unsafe impl<R: OsRuntime, W: OsWakerSet, U: UartPort> Sync
    for AsyncUartDriver<R, W, U>
{}

impl<R: OsRuntime, W: OsWakerSet, U: UartPort> fmt::Debug
    for AsyncUartDriver<R, W, U>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncUartDriver").finish_non_exhaustive()
    }
}

impl<R: OsRuntime, W: OsWakerSet, U: UartPort> AsyncUartDriver<R, W, U> {
    /// Create a new driver instance.
    ///
    /// The `uart` reference must be `'static` as it will be shared with
    /// spawned copier tasks that outlive the creating scope.
    pub const fn new(
        rx: RingBufRx<W>,
        tx: RingBufTx<W>,
        uart: &'static U,
    ) -> Self {
        Self {
            rx,
            tx,
            uart,
            #[cfg(feature = "telemetry")]
            telemetry: crate::async_::telemetry::Telemetry::new(),
            _runtime: PhantomData,
        }
    }

    /// Get a reference to the telemetry counters (only available with `telemetry` feature).
    #[cfg(feature = "telemetry")]
    pub const fn telemetry(&self) -> &crate::async_::telemetry::Telemetry {
        &self.telemetry
    }

    /// Start the RX copier task.
    ///
    /// Spawns an async task that continuously reads from the UART and
    /// pushes data into the RX ring buffer. Uses NAPI-style interrupt
    /// coalescing for high throughput.
    ///
    /// `enable_rx_intr` is called to re-enable RX interrupts after data
    /// has been consumed (or when the NAPI counter resets).
    pub fn start_rx_copier(&'static self, enable_rx_intr: fn()) {
        R::spawn(
            async move {
                self.rx_copier_loop(enable_rx_intr).await;
            },
            "uart-rx-copier",
        );
    }

    /// Start the TX copier task.
    ///
    /// Spawns an async task that continuously pops from the TX ring
    /// buffer and writes data to the UART.
    ///
    /// `enable_tx_intr` is called to re-enable TX interrupts when the
    /// UART's transmit holding register is full.
    pub fn start_tx_copier(&'static self, enable_tx_intr: fn()) {
        R::spawn(
            async move {
                self.tx_copier_loop(enable_tx_intr).await;
            },
            "uart-tx-copier",
        );
    }

    /// RX copier loop with NAPI interrupt coalescing.
    ///
    /// Continuously reads from the UART and pushes to the RX ring buffer.
    /// Tracks consecutive successful reads to implement NAPI-style
    /// interrupt coalescing:
    /// - Below threshold: read up to `COPIER_BUF_SIZE` bytes per iteration
    /// - At/above threshold: read up to `NAPI_BATCH_SIZE` bytes per
    ///   iteration (smaller batches for lower latency)
    /// - On no data: reset counter and re-enable RX interrupts
    async fn rx_copier_loop(&self, enable_rx_intr: fn()) {
        let mut read_buf = [0u8; COPIER_BUF_SIZE];
        let mut consecutive = 0u32;

        loop {
            poll_fn(|cx| {
                let batch = if consecutive >= NAPI_THRESHOLD {
                    NAPI_BATCH_SIZE
                } else {
                    COPIER_BUF_SIZE
                };

                let total = self.uart.receive_bytes(&mut read_buf[..batch]);

                if total > 0 {
                    self.rx.push_batch(&read_buf[..total]);
                }

                // NAPI logic: track consecutive successful reads
                if consecutive >= NAPI_THRESHOLD {
                    if total > 0 {
                        consecutive += 1;
                    } else {
                        consecutive = 0;
                        enable_rx_intr();
                    }
                } else {
                    consecutive =
                        if total > 0 { consecutive + 1 } else { 0 };
                }

                if consecutive < NAPI_THRESHOLD {
                    enable_rx_intr();
                }

                // Register waker for next interrupt
                RX_WAKER.register(cx.waker());

                if total > 0 {
                    Poll::Ready(total)
                } else {
                    Poll::Pending
                }
            })
            .await;
        }
    }

    /// TX copier loop.
    ///
    /// Continuously pops data from the TX ring buffer and sends it to
    /// the UART. When the UART's transmit holding register is full,
    /// enables TX interrupts and waits for the next opportunity.
    async fn tx_copier_loop(&self, enable_tx_intr: fn()) {
        let mut write_buf = [0u8; COPIER_BUF_SIZE];
        let mut pending = 0usize;
        let mut cursor = 0usize;

        loop {
            poll_fn(|cx| {
                #[cfg(feature = "telemetry")]
                self.telemetry.tx_poll.fetch_add(1, Ordering::Relaxed);

                // If we've sent all pending data, get more from ring buffer
                if cursor >= pending {
                    pending = self.tx.pop_batch(&mut write_buf);
                    cursor = 0;
                    if pending == 0 {
                        self.tx.register_waker(cx.waker());
                        return Poll::Pending;
                    }
                }

                // Send data to UART
                let sent =
                    self.uart.send_bytes(&write_buf[cursor..pending]);
                cursor += sent;

                #[cfg(feature = "telemetry")]
                if sent > 0 {
                    self.telemetry
                        .tx_hw_bytes
                        .fetch_add(sent as u64, Ordering::Relaxed);
                } else {
                    self.telemetry
                        .tx_no_progress
                        .fetch_add(1, Ordering::Relaxed);
                }

                // If we couldn't send everything, enable TX interrupt
                // for the next opportunity
                if cursor < pending {
                    enable_tx_intr();
                }

                // Register waker for next interrupt
                TX_WAKER.register(cx.waker());

                Poll::Ready(())
            })
            .await;
        }
    }
}
