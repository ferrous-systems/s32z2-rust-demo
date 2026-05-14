//! Generic Timer example for NXP S32Z280

#![no_std]
#![no_main]

use aarch32_cpu::generic_timer::GenericTimer;
use arm_dcc::dprintln as println;

/// The entry-point to the Rust application
#[aarch32_rt::entry]
fn main() -> ! {
    s32z2_rust_demo::setup_core();

    let peripherals = unsafe { s32z2_rust_demo::Peripherals::steal() };
    let cntfrq = aarch32_cpu::register::Cntfrq::read().0;
    println!("cntfrq = {:.03} MHz", cntfrq as f32 / 1_000_000.0);

    let delay_ticks = cntfrq * 2;

    let mut pgt = peripherals.physical_timer;
    let mut vgt = peripherals.virtual_timer;

    loop {
        let pgt_ref: &mut dyn GenericTimer = &mut pgt;
        let vgt_ref: &mut dyn GenericTimer = &mut vgt;
        for (timer, name) in [(pgt_ref, "physical"), (vgt_ref, "virtual")] {
            println!("Using {} timer ************************", name);

            println!("Print five, one per second...");
            for i in 0..5 {
                println!("i = {}", i);
                timer.delay_ms(1000);
            }

            let now = timer.counter();
            println!("{} is now: {}", name, now);
            println!("Waiting for {} {} ticks to count up...", delay_ticks, name);
            timer.counter_compare_set(now + delay_ticks as u64);
            timer.enable(true);
            while !timer.interrupt_status() {
                core::hint::spin_loop();
            }
            println!("Matched! {} count now {}", name, timer.counter());

            println!(
                "Waiting for {} {} ticks to count down...",
                delay_ticks, name
            );
            timer.countdown_set(delay_ticks);
            while !timer.interrupt_status() {
                core::hint::spin_loop();
            }
            println!(
                "{} countdown hit zero! (and is now {})",
                name,
                timer.countdown() as i32
            );
        }
    }
}
