//! Generic Timer example for NXP S32Z280

#![no_std]
#![no_main]

use aarch32_cpu::generic_timer::{El1PhysicalTimer, El1VirtualTimer, GenericTimer};
use arm_dcc::dprintln as println;

use s32z2_rust_demo::Peripherals;

/// The entry-point to the Rust application.
///
/// It is called by the start-up code in `lib.rs`
#[no_mangle]
pub fn s32z2_main(_peripherals: Peripherals) {
    println!("-- Running the 'generic_timer' binary on the NXP S32Z2 --");

    let cntfrq = aarch32_cpu::register::Cntfrq::read().0;
    println!("cntfrq = {:.03} MHz", cntfrq as f32 / 1_000_000.0);

    let delay_ticks = cntfrq * 2;

    let mut pgt = unsafe { El1PhysicalTimer::new() };
    let mut vgt = unsafe { El1VirtualTimer::new() };

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
