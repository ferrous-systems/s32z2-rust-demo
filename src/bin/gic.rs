//! GICv3 example for NXP S32Z280

#![no_std]
#![no_main]

use arm_gic::{
    gicv3::{GicCpuInterface, Group, SgiTarget, SgiTargetGroup},
    IntId, InterruptGroup,
};
use defmt::println;

/// The entry-point to the Rust application.
#[aarch32_rt::entry]
fn main() -> ! {
    s32z2_rust_demo::setup_core();

    let mut peripherals = unsafe { s32z2_rust_demo::Peripherals::steal() };

    // Configure a Software Generated Interrupt for Core 0
    println!("Configure SGI...");
    let sgi_intid = IntId::sgi(3);
    peripherals
        .gic
        .set_interrupt_priority(sgi_intid, Some(0), 0x31)
        .expect("set prio on SGI int");
    peripherals
        .gic
        .set_group(sgi_intid, Some(0), Group::Group1NS)
        .expect("set group on SGI int");

    println!("gic.enable_interrupt()");
    peripherals
        .gic
        .enable_interrupt(sgi_intid, Some(0), true)
        .expect("enabling SGI Int");

    println!("Enabling interrupts...");
    dump_cpsr();
    unsafe {
        aarch32_cpu::interrupt::enable();
    }
    dump_cpsr();

    // Send it
    println!("Send SGI");
    GicCpuInterface::send_sgi(
        sgi_intid,
        SgiTarget::List {
            affinity3: 0,
            affinity2: 0,
            affinity1: 0,
            target_list: 0b1,
        },
        SgiTargetGroup::CurrentGroup1,
    )
    .expect("send SGI");

    loop {
        aarch32_cpu::asm::nop();
    }
}

fn dump_cpsr() {
    let cpsr = aarch32_cpu::register::Cpsr::read();
    println!("CPSR: {:?}", cpsr);
}

/// Called when the Arm core gets an IRQ
#[aarch32_rt::irq]
fn irq_handler() {
    println!("> IRQ");
    while let Some(int_id) = GicCpuInterface::get_and_acknowledge_interrupt(InterruptGroup::Group1)
    {
        println!("- IRQ handle {}", int_id.raw_value());
        GicCpuInterface::end_interrupt(int_id, InterruptGroup::Group1);
    }
    println!("< IRQ");
}
