//! Registers example for Arm Cortex-R

#![no_std]
#![no_main]

// pull in our start-up code
use s32z2_rust_demo as _;

use arm_dcc::dprintln as println;

/// The entry-point to the Rust application.
///
/// It is called by the start-up code in `lib.rs`
#[no_mangle]
pub fn s32z2_main() {
    println!("{:?}", aarch32_cpu::register::Midr::read());
    println!("{:?}", aarch32_cpu::register::Cpsr::read());
    println!("{:?}", aarch32_cpu::register::ImpCbar::read());
    println!("{:?}", aarch32_cpu::register::Vbar::read());
    // This only works in EL2 and start-up put us in EL1
    // println!("{:?}", aarch32_cpu::register::Hvbar::read());

    println!(
        "Sys Stack: {:08x?}",
        aarch32_rt::stacks::Stack::Sys.range(0)
    );

    println!(
        "{:?} before setting C, I and Z",
        aarch32_cpu::register::Sctlr::read()
    );
    aarch32_cpu::register::Sctlr::modify(|w| {
        w.set_c(true);
        w.set_i(true);
        w.set_z(true);
    });
    println!("{:?} after", aarch32_cpu::register::Sctlr::read());
}
