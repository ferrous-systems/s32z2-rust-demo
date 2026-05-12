//! An embassy-time implementation for Arm Generic Timer with multiple cores

use core::{cell::RefCell, task::Waker};

use aarch32_cpu::generic_timer::{self, GenericTimer as _};
use embassy_sync::blocking_mutex::{raw::CriticalSectionRawMutex, Mutex};

use super::cpuid;

/// A type for handling a queue of alarms on the EL1 Virtual Timer
struct Aarch32VirtualTimerQueue {
    core0: Mutex<CriticalSectionRawMutex, RefCell<Aarch32VirtualTimerQueueInner>>,
    core1: Mutex<CriticalSectionRawMutex, RefCell<Aarch32VirtualTimerQueueInner>>,
}

impl embassy_time_driver::Driver for Aarch32VirtualTimerQueue {
    fn now(&self) -> u64 {
        generic_timer::read_virtual_timer()
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        let cpuid = cpuid();
        if cpuid == 0 {
            critical_section::with(|cs| {
                let mut inner = self.core0.borrow(cs).borrow_mut();
                inner.schedule_wake(at, waker);
            });
        } else {
            critical_section::with(|cs| {
                let mut inner = self.core1.borrow(cs).borrow_mut();
                inner.schedule_wake(at, waker);
            });
        }
    }
}

impl Aarch32VirtualTimerQueue {
    /// Call this from the interrupt handler when it goes off
    fn on_irq(&self) {
        if cpuid() == 0 {
            critical_section::with(|cs| {
                let mut inner = self.core0.borrow(cs).borrow_mut();
                inner.update_alarm();
            });
        } else {
            critical_section::with(|cs| {
                let mut inner = self.core1.borrow(cs).borrow_mut();
                inner.update_alarm();
            });
        }
    }
}

/// Call this from the interrupt handler when VIRTUAL_TIMER_PPI fires
pub fn timer_irq() {
    DRIVER.on_irq();
}

/// Mutable state for our alarm queue
struct Aarch32VirtualTimerQueueInner {
    queue: embassy_time_queue_utils::Queue,
}

impl Aarch32VirtualTimerQueueInner {
    /// Schedule a wake-up for the next thing in the queue
    fn schedule_wake(&mut self, at: u64, waker: &Waker) {
        if self.queue.schedule_wake(at, waker) {
            // alarm needs updating
            self.update_alarm();
        }
    }

    /// Check the time, and the queue, and maybe set an alarm (or turn it off)
    fn update_alarm(&mut self) {
        let now = generic_timer::read_virtual_timer();
        let next = self.queue.next_expiration(now);

        // SAFETY: we have &mut on this timer driver, and it's the only thing that owns
        // a timer.
        let mut vt = unsafe { generic_timer::El1VirtualTimer::new() };
        if next == u64::MAX {
            // turn the timer interrupt off
            vt.interrupt_mask(true);
        } else {
            // set an alarm - will fire instantly if it's in the past
            vt.counter_compare_set(next);
            vt.interrupt_mask(false);
            vt.enable(true);
        }
    }
}

embassy_time_driver::time_driver_impl!(static DRIVER: Aarch32VirtualTimerQueue = Aarch32VirtualTimerQueue {
    core0: Mutex::new(RefCell::new(
        Aarch32VirtualTimerQueueInner {
            queue: embassy_time_queue_utils::Queue::new(),
        }
    )),
    core1: Mutex::new(RefCell::new(
        Aarch32VirtualTimerQueueInner {
            queue: embassy_time_queue_utils::Queue::new(),
        }
    ))
});
