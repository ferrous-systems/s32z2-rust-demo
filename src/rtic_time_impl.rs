//! Implmentation of the rtic-time traits

static TIMER_QUEUE: rtic_time::timer_queue::TimerQueue<TimerBackend> =
    rtic_time::timer_queue::TimerQueue::new();

pub struct TimerBackend;

impl rtic_time::timer_queue::TimerQueueBackend for TimerBackend {
    type Ticks = u64;

    fn now() -> Self::Ticks {
        aarch32_cpu::generic_timer::read_virtual_timer()
    }

    fn set_compare(instant: Self::Ticks) {
        use aarch32_cpu::generic_timer::GenericTimer;
        let mut timer = unsafe { aarch32_cpu::generic_timer::El1VirtualTimer::new() };
        timer.counter_compare_set(instant);
    }

    fn clear_compare_flag() {
        use aarch32_cpu::generic_timer::GenericTimer;
        let mut timer = unsafe { aarch32_cpu::generic_timer::El1VirtualTimer::new() };
        timer.counter_compare_set(u64::MAX);
    }

    fn pend_interrupt() {
        use aarch32_cpu::generic_timer::GenericTimer;
        let mut timer = unsafe { aarch32_cpu::generic_timer::El1VirtualTimer::new() };
        timer.counter_compare_set(0);
    }

    fn timer_queue() -> &'static rtic_time::timer_queue::TimerQueue<Self> {
        &TIMER_QUEUE
    }
}

pub struct Mono;

impl Mono {
    pub fn start(mut timer: aarch32_cpu::generic_timer::El1VirtualTimer) {
        use aarch32_cpu::generic_timer::GenericTimer;
        TIMER_QUEUE.initialize(TimerBackend {});
        timer.enable(true);
        timer.interrupt_mask(false);
    }

    pub fn handle_irq() {
        unsafe {
            TIMER_QUEUE.on_monotonic_interrupt();
        }
    }
}

impl rtic_time::monotonic::TimerQueueBasedMonotonic for Mono {
    type Backend = TimerBackend;

    type Instant = Instant;

    type Duration = Duration;
}

rtic_time::impl_embedded_hal_delay_fugit!(Mono);
rtic_time::impl_embedded_hal_async_delay_fugit!(Mono);

pub type Instant = fugit::Instant<u64, 1, 8_000_000>;
pub type Duration = fugit::Duration<u64, 1, 8_000_000>;
