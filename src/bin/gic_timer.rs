//! GICv3 + Generic Timer example for NXP S32Z280

#![no_std]
#![no_main]

use core::ptr::NonNull;

use arm_dcc::dprintln as println;
use arm_gic::{
    gicv3::{GicCpuInterface, GicV3, Group, InterruptGroup, SgiTarget, SgiTargetGroup},
    IntId, UniqueMmioPointer,
};
use cortex_ar::generic_timer::{El1VirtualTimer, GenericTimer};

// pull in our start-up code
use s32z2_rust_demo as _;

/// Offset from PERIPHBASE for GIC Distributor
const GICD_BASE_OFFSET: usize = 0x0000_0000usize;

/// Offset from PERIPHBASE for the first GIC Redistributor
const GICR_BASE_OFFSET: usize = 0x0010_0000usize;

/// The PPI for the virutal timer, according to the Cortex-R52 Reference Manual
///
/// This corresponds to Interrupt ID 27.
const VIRTUAL_TIMER_PPI: IntId = IntId::ppi(11);

/// Our software interrupt ID
const SGI_ID: IntId = IntId::sgi(3);

/// The entry-point to the Rust application.
///
/// It is called by the start-up code in `lib.rs`
#[no_mangle]
pub fn s32z2_main() {
    // Get the GIC address by reading CBAR
    let periphbase = cortex_ar::register::ImpCbar::read().periphbase();
    println!("Found PERIPHBASE {:010p}", periphbase);
    let gicd_base = periphbase.wrapping_byte_add(GICD_BASE_OFFSET);
    let gicr_base = periphbase.wrapping_byte_add(GICR_BASE_OFFSET);

    // Initialise the GIC.
    println!(
        "Creating GIC driver @ {:010p} / {:010p}",
        gicd_base, gicr_base
    );
    let gicd = unsafe { UniqueMmioPointer::new(NonNull::new(gicd_base.cast()).unwrap()) };
    let gicr_base = NonNull::new(gicr_base.cast()).unwrap();
    let mut gic: GicV3 = unsafe { GicV3::new(gicd, gicr_base, 1, false) };
    println!("Calling git.setup(0)");
    gic.setup(0);
    GicCpuInterface::set_priority_mask(0x80);

    // Configure a Software Generated Interrupt for Core 0
    println!("Configure SGI...");
    gic.set_interrupt_priority(SGI_ID, Some(0), 0x31)
        .expect("SGI set_interrupt_priority");
    gic.set_group(SGI_ID, Some(0), Group::Group1NS)
        .expect("SGI set_group");
    gic.enable_interrupt(SGI_ID, Some(0), true)
        .expect("SGI enable_interrupt");

    println!("Configure Timer Interrupt...");
    gic.set_interrupt_priority(VIRTUAL_TIMER_PPI, Some(0), 0x31)
        .expect("Timer set_interrupt_priority");
    gic.set_group(VIRTUAL_TIMER_PPI, Some(0), Group::Group1NS)
        .expect("Timer set_group");
    gic.enable_interrupt(VIRTUAL_TIMER_PPI, Some(0), true)
        .expect("Timer enable_interrupt");

    let mut vgt = unsafe { El1VirtualTimer::new() };
    vgt.enable(true);
    vgt.interrupt_mask(false);
    vgt.counter_compare_set(u64::MAX);

    println!("Enabling interrupts...");
    dump_cpsr();
    unsafe {
        cortex_ar::interrupt::enable();
    }
    dump_cpsr();

    // Send it
    println!("Send SGI");
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

    vgt.countdown_set(vgt.frequency_hz());

    let mut count: u32 = 0;
    loop {
        cortex_ar::asm::wfi();
        println!("Main loop wake up {}", count);
        count = count.wrapping_add(1);
    }
}

fn dump_cpsr() {
    let cpsr = cortex_ar::register::Cpsr::read();
    println!("CPSR: {:?}", cpsr);
}

/// Called when the Arm core gets an IRQ
#[cortex_r_rt::irq]
fn irq_handler() {
    println!("> IRQ");
    while let Some(int_id) = GicCpuInterface::get_and_acknowledge_interrupt(InterruptGroup::Group1)
    {
        println!("- IRQ handle {:?}", int_id);
        if int_id == VIRTUAL_TIMER_PPI {
            handle_timer_irq();
        } else if int_id == SGI_ID {
            handle_sgi_irq();
        }
        GicCpuInterface::end_interrupt(int_id, InterruptGroup::Group1);
    }
    println!("< IRQ");
}

/// Run when the timer IRQ fires
fn handle_timer_irq() {
    println!("- Timer fired - resetting");
    // trigger a timer in 1 second
    let mut vgt = unsafe { El1VirtualTimer::new() };
    vgt.countdown_set(vgt.countdown().wrapping_add(vgt.frequency_hz()));
}

/// Run when the SGI is fired
fn handle_sgi_irq() {
    println!("- SGI fired");
}
