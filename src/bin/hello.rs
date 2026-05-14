//! Semihosting hello-world for NXP S32Z2

#![no_std]
#![no_main]

use defmt::println;

/// The entry-point to the Rust application.
#[aarch32_rt::entry]
fn main() -> ! {
    s32z2_rust_demo::setup_core();

    let _peripherals = unsafe { s32z2_rust_demo::Peripherals::steal() };

    let x = 1.0f64;
    let y = x * 2.0;
    println!("Hello, this is semihosting! x = {=f64}, y = {=f64}", x, y);
    panic!("I am an example panic");
}
