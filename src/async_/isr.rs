// SPDX-License-Identifier: MIT OR Apache-2.0

//! UART 16550 ISR handler with AtomicWaker pattern.
//!
//! Provides a minimal ISR handler that reads the ISR register, disables the
//! corresponding interrupt, and wakes the appropriate async waker.

use core::ptr::NonNull;
use core::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use embassy_sync::waitqueue::AtomicWaker;

use crate::spec::registers::{InterruptType, IER, ISR, LSR, offsets};

/// RX data ready waker — woken when data arrives.
pub static RX_WAKER: AtomicWaker = AtomicWaker::new();

/// TX buffer empty waker — woken when THR is empty.
pub static TX_WAKER: AtomicWaker = AtomicWaker::new();

/// Drain complete waker — woken when transmitter is fully empty (for `tcdrain`).
pub static DRAIN_WAKER: AtomicWaker = AtomicWaker::new();

/// IRQ invocation counter — useful for benchmarking and diagnostics.
pub static IRQ_COUNT: AtomicU64 = AtomicU64::new(0);

/// Returns the total number of ISR invocations since boot.
pub fn irq_count() -> u64 {
    IRQ_COUNT.load(Ordering::Relaxed)
}

/// Lock-free ISR register access — safe in ISR context.
pub(crate) struct IsrRegisters {
    base: NonNull<u8>,
}

impl IsrRegisters {
    /// Create a new ISR register accessor.
    ///
    /// # Safety
    ///
    /// `base` must be a valid UART MMIO base address.
    pub(crate) const unsafe fn new(base: NonNull<u8>) -> Self {
        Self { base }
    }

    /// Read the ISR register (offset 2, stride 1).
    ///
    /// # Safety
    ///
    /// The base address must be valid and the ISR register must be mapped.
    pub(crate) unsafe fn read_isr(&self) -> ISR {
        // SAFETY: Caller guarantees base is valid and ISR is mapped.
        unsafe {
            let ptr = self.base.as_ptr().add(offsets::ISR);
            ISR::from_bits_retain(ptr.read_volatile())
        }
    }

    /// Read the LSR register (offset 5, stride 1).
    ///
    /// # Safety
    ///
    /// The base address must be valid and the LSR register must be mapped.
    pub(crate) unsafe fn read_lsr(&self) -> LSR {
        // SAFETY: Caller guarantees base is valid and LSR is mapped.
        unsafe {
            let ptr = self.base.as_ptr().add(offsets::LSR);
            LSR::from_bits_retain(ptr.read_volatile())
        }
    }

    /// Disable RX interrupt (clear `DATA_READY` bit in IER).
    ///
    /// # Safety
    ///
    /// The base address must be valid and the IER register must be mapped.
    pub(crate) unsafe fn disable_rx_intr(&self, cached_ier: &AtomicU8) {
        let current = cached_ier.load(Ordering::Relaxed);
        let new_val = current & !IER::DATA_READY.bits();
        cached_ier.store(new_val, Ordering::Relaxed);
        // SAFETY: Caller guarantees base is valid and IER is mapped.
        unsafe {
            let ptr = self.base.as_ptr().add(offsets::IER);
            ptr.write_volatile(new_val);
        }
    }

    /// Disable TX interrupt (clear `THR_EMPTY` bit in IER).
    ///
    /// # Safety
    ///
    /// The base address must be valid and the IER register must be mapped.
    pub(crate) unsafe fn disable_tx_intr(&self, cached_ier: &AtomicU8) {
        let current = cached_ier.load(Ordering::Relaxed);
        let new_val = current & !IER::THR_EMPTY.bits();
        cached_ier.store(new_val, Ordering::Relaxed);
        // SAFETY: Caller guarantees base is valid and IER is mapped.
        unsafe {
            let ptr = self.base.as_ptr().add(offsets::IER);
            ptr.write_volatile(new_val);
        }
    }
}

/// UART ISR handler — called from IRQ context.
///
/// This handler is minimal (ISR principle):
/// 1. Read ISR to determine interrupt type
/// 2. Disable the corresponding interrupt
/// 3. Wake the appropriate waker
/// 4. Return immediately
pub fn uart_isr_handler(_irq: usize, base: NonNull<u8>, cached_ier: &AtomicU8) {
    IRQ_COUNT.fetch_add(1, Ordering::Relaxed);
    // SAFETY: Called from ISR context with a valid base address.
    unsafe {
        let regs = IsrRegisters::new(base);
        let isr = regs.read_isr();

        match isr.interrupt_type() {
            Some(InterruptType::ReceivedDataReady)
            | Some(InterruptType::ReceptionTimeout) => {
                regs.disable_rx_intr(cached_ier);
                RX_WAKER.wake();
            }
            Some(InterruptType::TransmitterHoldingRegisterEmpty) => {
                regs.disable_tx_intr(cached_ier);
                TX_WAKER.wake();
                if regs.read_lsr().contains(LSR::TRANSMITTER_EMPTY) {
                    DRAIN_WAKER.wake();
                }
            }
            _ => {}
        }
    }
}
