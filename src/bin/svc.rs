//! SVC (Service Call) example for NXP S32Z2

#![no_std]
#![no_main]

use s32z2_rust_demo::Peripherals;

use arm_dcc::dprintln as println;

/// The entry-point to the Rust application.
///
/// It is called by the start-up code in `lib.rs`
#[no_mangle]
pub fn s32z2_main(_peripherals: Peripherals) {
    println!("-- Running the 'svc' binary on the NXP S32Z2 --");

    let x = 1;
    let y = x + 1;
    let z = (y as f64) * 1.5;
    println!("x = {}, y = {}, z = {:0.3}", x, y, z);
    let semihosting_result = aarch32_cpu::svc!(0xABCDEF);
    println!("semihosting result was 0x{:04x}", semihosting_result);
    println!("x = {}, y = {}, z = {:0.3}", x, y, z);
    panic!("I am an example panic");
}

/// This is our SVC exception handler
#[aarch32_rt::exception(SupervisorCall)]
fn svc_handler(arg: u32, frame: &aarch32_rt::Frame) -> u32 {
    println!(
        "In SupervisorCall handler, with arg={:#06x}, frame={:08x?}",
        arg, frame
    );
    if arg == 0xABCDEF {
        // test nested SVC calls
        aarch32_cpu::svc!(0x456789);
    }
    0x1234
}
