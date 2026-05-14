//! SVC (Service Call) example for NXP S32Z2

#![no_std]
#![no_main]

use aarch32_cpu as _;

/// The entry-point to the Rust application.
#[aarch32_rt::entry]
fn main() -> ! {
    s32z2_rust_demo::setup_core();

    let _peripherals = unsafe { s32z2_rust_demo::Peripherals::steal() };

    let x = 1;
    let y = x + 1;
    let z = (y as f64) * 1.5;
    defmt::info!("x = {=i32}, y = {=i32}, z = {=f64}", x, y, z);
    let semihosting_result = aarch32_cpu::svc!(0xABCDEF);
    defmt::info!("semihosting result was 0x{:04x}", semihosting_result);
    defmt::info!("x = {=i32}, y = {=i32}, z = {=f64}", x, y, z);
    panic!("I am an example panic");
}

/// This is our SVC exception handler
#[aarch32_rt::exception(SupervisorCall)]
fn svc_handler(arg: u32, frame: &aarch32_rt::Frame) -> u32 {
    defmt::info!(
        "In SupervisorCall handler, with arg={:x}, frame={}",
        arg,
        defmt::Debug2Format(frame)
    );
    if arg == 0xABCDEF {
        // test nested SVC calls
        aarch32_cpu::svc!(0x456789);
    }
    0x1234
}
