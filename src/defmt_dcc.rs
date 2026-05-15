use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

#[defmt::global_logger]
struct Logger;

static DCC_ENCODER: DccEncoder = DccEncoder::new();

struct DccEncoder {
    /// A boolean lock
    ///
    /// Is `true` when `acquire` has been called and we have exclusive access to
    /// the rest of this structure.
    taken: AtomicBool,
    /// We need to remember this to exit a critical section
    cs_restore: UnsafeCell<critical_section::RestoreState>,
    /// A defmt::Encoder for encoding frames
    encoder: UnsafeCell<defmt::Encoder>,
}

impl DccEncoder {
    /// Create a new semihosting-based defmt-encoder
    const fn new() -> DccEncoder {
        DccEncoder {
            taken: AtomicBool::new(false),
            cs_restore: UnsafeCell::new(critical_section::RestoreState::invalid()),
            encoder: UnsafeCell::new(defmt::Encoder::new()),
        }
    }

    /// Acquire the defmt encoder.
    fn acquire(&self) {
        // safety: Must be paired with corresponding call to release(), see below
        let restore = unsafe { critical_section::acquire() };

        // NB: You can re-enter critical sections but we need to make sure
        // no-one does that.
        if self.taken.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly")
        }

        // no need for CAS because we are in a critical section
        self.taken.store(true, Ordering::Relaxed);

        // safety: accessing the cell is OK because we have acquired a critical
        // section.
        unsafe {
            self.cs_restore.get().write(restore);
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            encoder.start_frame(|b| {
                arm_dcc::write_all(b);
            });
        }
    }

    /// Release the defmt encoder.
    unsafe fn release(&self) {
        if !self.taken.load(Ordering::Relaxed) {
            panic!("defmt release out of context")
        }

        // safety: accessing the cell is OK because we have acquired a critical
        // section.
        unsafe {
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            encoder.end_frame(|b| {
                arm_dcc::write_all(b);
            });
            let restore = self.cs_restore.get().read();
            self.taken.store(false, Ordering::Relaxed);
            // paired with exactly one acquire call
            critical_section::release(restore);
        }
    }

    /// Write bytes to the defmt encoder.
    unsafe fn write(&self, bytes: &[u8]) {
        if !self.taken.load(Ordering::Relaxed) {
            panic!("defmt write out of context")
        }

        // safety: accessing the cell is OK because we have acquired a critical
        // section.
        unsafe {
            let encoder: &mut defmt::Encoder = &mut *self.encoder.get();
            encoder.write(bytes, |b| {
                arm_dcc::write_all(b);
            });
        }
    }
}

unsafe impl Sync for DccEncoder {}

unsafe impl defmt::Logger for Logger {
    fn acquire() {
        DCC_ENCODER.acquire();
    }

    unsafe fn flush() {
        // Do nothing.
        //
        // semihosting is fundamentally blocking, and does not have I/O buffers the target can control.
        // After write returns, the host has the data, so there's nothing left to flush.
    }

    unsafe fn release() {
        unsafe {
            DCC_ENCODER.release();
        }
    }

    unsafe fn write(bytes: &[u8]) {
        unsafe {
            DCC_ENCODER.write(bytes);
        }
    }
}
