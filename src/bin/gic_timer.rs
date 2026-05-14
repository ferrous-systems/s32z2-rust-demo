//! GICv3 + Generic Timer example for NXP S32Z280

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

use aarch32_cpu::generic_timer::{El1VirtualTimer, GenericTimer};
use arm_dcc::dprintln as println;
use arm_gic::{
    gicv3::{GicCpuInterface, Group, InterruptGroup, SgiTarget, SgiTargetGroup},
    IntId,
};

/// The PPI for the virutal timer, according to the Cortex-R52 Reference Manual
///
/// This corresponds to Interrupt ID 27.
const VIRTUAL_TIMER_PPI: IntId = IntId::ppi(11);

/// Our software interrupt ID
const SGI_ID: IntId = IntId::sgi(3);

/// Just a dummy number that Core 1 will increment in a loop
pub static CORE1_COUNTER: AtomicU32 = AtomicU32::new(0);

/// The entry-point to the Rust application for Core 0.
#[aarch32_rt::entry]
fn main() -> ! {
    s32z2_rust_demo::setup_core();

    let mut peripherals = unsafe { s32z2_rust_demo::Peripherals::steal() };

    println!("Configure SGI...");
    // this is higher priority than the timer
    peripherals
        .gic
        .set_interrupt_priority(SGI_ID, Some(0), 0x10)
        .expect("SGI set_interrupt_priority");
    peripherals
        .gic
        .set_group(SGI_ID, Some(0), Group::Group1NS)
        .expect("SGI set_group");
    peripherals
        .gic
        .enable_interrupt(SGI_ID, Some(0), true)
        .expect("SGI enable_interrupt");

    println!("Configure Timer Interrupt...");
    // this is lower priority than the SGI, so they will nest
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

    peripherals.virtual_timer.enable(true);
    peripherals.virtual_timer.interrupt_mask(false);
    peripherals.virtual_timer.counter_compare_set(u64::MAX);

    println!("Enabling interrupts...");
    unsafe {
        aarch32_cpu::interrupt::enable();
    }

    println!("Waking core 1...");
    s32z2_rust_demo::wake_core1();

    peripherals
        .virtual_timer
        .countdown_set(peripherals.virtual_timer.frequency_hz());

    let mut count: u32 = 0;
    loop {
        aarch32_cpu::asm::wfi();
        println!(
            "Main loop wake up {}, core1 counter {}",
            count,
            CORE1_COUNTER.load(core::sync::atomic::Ordering::Relaxed)
        );
        count = count.wrapping_add(1);
    }
}

/// Called when the Arm core gets an IRQ
#[aarch32_rt::irq]
fn irq_handler() {
    println!("> irq_handler()");
    while let Some(int_id) = GicCpuInterface::get_and_acknowledge_interrupt(InterruptGroup::Group1)
    {
        println!("- Handling {:?}", int_id);
        // Re-enable interrupts
        //
        // NB: Don't do this until after you've talked to the GIC
        //
        // Safety: not in a critical section, so safe to enable interrupts
        unsafe {
            aarch32_cpu::interrupt::enable();
        }
        if int_id == VIRTUAL_TIMER_PPI {
            handle_timer_irq();
        } else if int_id == SGI_ID {
            handle_sgi_irq();
        }
        aarch32_cpu::interrupt::disable();
        println!("- Handled {:?}", int_id);
        GicCpuInterface::end_interrupt(int_id, InterruptGroup::Group1);
    }
    println!("< irq_handler()");
}

/// Run when the timer IRQ fires
fn handle_timer_irq() {
    println!("> handle_timer_irq()");

    println!("--- Resetting timer...");

    // trigger a timer in 1 second
    let mut vgt = unsafe { El1VirtualTimer::new() };
    vgt.countdown_set(vgt.countdown().wrapping_add(vgt.frequency_hz()));

    println!("--- Sending SGI...");
    GicCpuInterface::send_sgi(
        SGI_ID,
        SgiTarget::List {
            affinity3: 0,
            affinity2: 0,
            affinity1: 0,
            target_list: 0b1,
        },
        SgiTargetGroup::CurrentGroup1,
    )
    .expect("send sgi");
    // make sure the SGI happens
    aarch32_cpu::asm::isb();

    println!("< handle_timer_irq()");
}

/// Run when the SGI is fired
fn handle_sgi_irq() {
    println!("- handle_sgi_irq()");
}

/// The entry-point to the Rust application for Core 1.
///
/// It is called by the start-up code in `lib.rs`
#[unsafe(no_mangle)]
pub extern "C" fn kmain2() {
    s32z2_rust_demo::setup_core();
    loop {
        CORE1_COUNTER.fetch_add(1, Relaxed);
    }
}
