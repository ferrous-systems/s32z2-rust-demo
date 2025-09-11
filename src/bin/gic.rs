//! GICv3 example for NXP S32Z280

#![no_std]
#![no_main]

// pull in our start-up code
use s32z2_rust_demo as _;

use arm_dcc::dprintln as println;
use arm_gic::{
    gicv3::{GicV3, Group, InterruptGroup, SgiTarget, SgiTargetGroup},
    IntId,
};

/// The entry-point to the Rust application.
///
/// It is called by the start-up code in `lib.rs`
#[no_mangle]
pub fn s32z2_main() {
    if let Err(e) = main() {
        panic!("main returned {:?}", e);
    }
}

/// Offset from PERIPHBASE for GIC Distributor
const GICD_BASE_OFFSET: usize = 0x0000_0000usize;

/// Offset from PERIPHBASE for the first GIC Redistributor
const GICR_BASE_OFFSET: usize = 0x0010_0000usize;

fn dump_cpsr() {
    let cpsr = cortex_ar::register::Cpsr::read();
    println!("CPSR: {:?}", cpsr);
}

/// The main function of our Rust application.
///
/// Called by [`kmain`].
fn main() -> Result<(), core::fmt::Error> {
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
    let mut gic: GicV3 = unsafe { GicV3::new(gicd_base.cast(), gicr_base.cast(), 1, false) };
    println!("Calling git.setup(0)");
    gic.setup(0);
    GicV3::set_priority_mask(0x80);

    // Configure a Software Generated Interrupt for Core 0
    println!("Configure SGI...");
    let sgi_intid = IntId::sgi(3);
    gic.set_interrupt_priority(sgi_intid, Some(0), 0x31);
    gic.set_group(sgi_intid, Some(0), Group::Group1NS);

    println!("gic.enable_interrupt()");
    gic.enable_interrupt(sgi_intid, Some(0), true);

    println!("Enabling interrupts...");
    dump_cpsr();
    unsafe {
        cortex_ar::interrupt::enable();
    }
    dump_cpsr();

    // Send it
    println!("Send SGI");
    GicV3::send_sgi(
        sgi_intid,
        SgiTarget::List {
            affinity3: 0,
            affinity2: 0,
            affinity1: 0,
            target_list: 0b1,
        },
        SgiTargetGroup::CurrentGroup1,
    );

    loop {
        cortex_ar::asm::nop();
    }
}

/// Called when the Arm core gets an IRQ
#[cortex_r_rt::irq]
fn irq_handler() {
    println!("> IRQ");
    while let Some(int_id) = GicV3::get_and_acknowledge_interrupt(InterruptGroup::Group1) {
        println!("- IRQ handle {:?}", int_id);
        GicV3::end_interrupt(int_id, InterruptGroup::Group1);
    }
    println!("< IRQ");
}
