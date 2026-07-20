//! embassy hello-world for NXP S32Z2
//!
//! Runs two tasks - one prints once per second, the other every 3 seconds

#![no_std]
#![no_main]

use arm_dcc::dprintln as println;

use arm_gic::gicv3::Group;
use embassy_time::{Duration, Instant, Ticker};
use s32z2_rust_demo::{Peripherals, VIRTUAL_TIMER_PPI};

#[no_mangle]
pub fn s32z2_main(mut peripherals: Peripherals) -> ! {
    println!("Configure Timer Interrupt...");
    peripherals
        .gic
        .set_interrupt_priority(VIRTUAL_TIMER_PPI, Some(0), 0x20)
        .expect("Timer set_interrupt_priority");
    peripherals
        .gic
        .set_group(VIRTUAL_TIMER_PPI, Some(0), Group::Group1NS)
        .expect("Timer set_group");
    peripherals
        .gic
        .enable_interrupt(VIRTUAL_TIMER_PPI, Some(0), true)
        .expect("Timer enable_interrupt");

    println!("Enabling interrupts on Core 0...");
    unsafe {
        aarch32_cpu::interrupt::enable();
    }

    main()
}

/// This is the entry point to our embassy program
///
/// It spawns [second_task], then prints once per second.
#[embassy_executor::main()]
async fn main(spawner: embassy_executor::Spawner) -> ! {
    println!("-- Running the 'embassy_hello' binary on the NXP S32Z2 --");

    spawner.spawn(second_task().unwrap());

    let mut ticker = Ticker::every(Duration::from_secs(1));
    loop {
        println!(
            "++ Task One ++ @ {:.03}s",
            Instant::now().as_millis() as f64 / 1000.0
        );
        ticker.next().await;
    }
}

/// A second example task
///
/// It prints once every 3 seconds
#[embassy_executor::task()]
async fn second_task() -> ! {
    let mut ticker = Ticker::every(Duration::from_secs(3));
    loop {
        println!(
            "== Task Two == @ {:.03}s",
            Instant::now().as_millis() as f64 / 1000.0
        );
        ticker.next().await;
    }
}

#[aarch32_rt::irq]
fn irq_handler() {
    use arm_gic::gicv3::{GicCpuInterface, InterruptGroup};
    while let Some(int_id) = GicCpuInterface::get_and_acknowledge_interrupt(InterruptGroup::Group1)
    {
        match int_id {
            s32z2_rust_demo::VIRTUAL_TIMER_PPI => {
                s32z2_rust_demo::timer_irq();
            }
            _ => unreachable!("We handle all enabled IRQs"),
        }
        GicCpuInterface::end_interrupt(int_id, InterruptGroup::Group1);
    }
}
