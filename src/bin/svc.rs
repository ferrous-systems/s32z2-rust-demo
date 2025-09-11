//! SVC (Service Call) example for NXP S32Z2

#![no_std]
#![no_main]

// pull in our start-up code
use cortex_ar as _;
use s32z2_rust_demo as _;

use arm_dcc::dprintln as println;

/// The entry-point to the Rust application.
///
/// It is called by the start-up code in `lib.rs`
#[no_mangle]
pub fn s32z2_main() {
    let x = 1;
    let y = x + 1;
    let z = (y as f64) * 1.5;
    println!("x = {}, y = {}, z = {:0.3}", x, y, z);
    cortex_ar::svc!(0xABCDEF);
    println!("x = {}, y = {}, z = {:0.3}", x, y, z);
    panic!("I am an example panic");
}

/// This is our SVC exception handler
#[cortex_r_rt::exception(SupervisorCall)]
fn svc_handler(arg: u32) {
    println!("In SupervisorCall handler, with arg={:#06x}", arg);
    if arg == 0xABCDEF {
        // test nested SVC calls
        cortex_ar::svc!(0x456789);
    }
}
