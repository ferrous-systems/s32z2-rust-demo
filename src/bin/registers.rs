//! Registers example for Arm Cortex-R

#![no_std]
#![no_main]

// pull in our start-up code
use s32z2_rust_demo as _;

use arm_dcc::dprintln as println;

extern "C" {
    static _stack_top: u32;
}

/// The entry-point to the Rust application.
///
/// It is called by the start-up code in `lib.rs`
#[no_mangle]
pub fn s32z2_main() {
    println!("{:?}", cortex_ar::register::Midr::read());
    println!("{:?}", cortex_ar::register::Cpsr::read());
    println!("{:?}", cortex_ar::register::ImpCbar::read());
    println!("{:?}", cortex_ar::register::Vbar::read());
    // This only works in EL2 and start-up put us in EL1
    // println!("{:?}", cortex_ar::register::Hvbar::read());

    println!("_stack_top: {:010p}", core::ptr::addr_of!(_stack_top));

    println!(
        "{:?} before setting C, I and Z",
        cortex_ar::register::Sctlr::read()
    );
    cortex_ar::register::Sctlr::modify(|w| {
        w.set_c(true);
        w.set_i(true);
        w.set_z(true);
    });
    println!("{:?} after", cortex_ar::register::Sctlr::read());
}
