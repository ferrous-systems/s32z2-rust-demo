//! Semihosting hello-world for NXP S32Z2

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
    let x = 1.0f64;
    let y = x * 2.0;
    println!("Hello, this is semihosting! x = {:0.3}, y = {:0.3}", x, y);
    panic!("I am an example panic");
}
