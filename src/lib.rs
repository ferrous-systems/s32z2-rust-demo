//! Common code for all S32Z2 examples

#![no_std]

use core::{ptr::NonNull, sync::atomic::AtomicBool};

use aarch32_cpu::generic_timer::El1VirtualTimer;
use aarch32_rt as _;
use arm_dcc::dprintln as println;
use arm_gic::{
    gicv3::{GicCpuInterface, GicV3},
    IntId, UniqueMmioPointer,
};
use panic_dcc as _;

mod clocks;
mod embassy_time_impl;
mod mpu;

pub use embassy_time_impl::timer_irq;

/// Offset from PERIPHBASE for GIC Distributor
pub const GICD_BASE_OFFSET: usize = 0x0000_0000usize;

/// Offset from PERIPHBASE for the first GIC Redistributor
pub const GICR_BASE_OFFSET: usize = 0x0010_0000usize;

/// The PPI for the virtual timer, according to the Cortex-R52 Technical Reference Manual,
/// Table 10-3: PPI assignments.
///
/// This corresponds to Interrupt ID 27.
pub const VIRTUAL_TIMER_PPI: IntId = IntId::ppi(11);

/// Controls when Core 1 can start running.
///
/// Initialised to false, set to true once Core 0 is ready for Core 1 to start.
static CORE1_RELEASED: AtomicBool = AtomicBool::new(false);

/// The peripherals we give Core 0 on start-up
pub struct Peripherals {
    pub gic: GicV3<'static>,
    pub virtual_timer: El1VirtualTimer,
}

/// The entry-point to the Rust application on Core 0
#[aarch32_rt::entry]
fn kmain() -> ! {
    unsafe extern "Rust" {
        safe fn s32z2_main(peripherals: Peripherals);
    }
    setup_core();

    // Get the GIC address by reading CBAR
    let periphbase = aarch32_cpu::register::ImpCbar::read().periphbase();
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

    let peripherals = Peripherals {
        gic,
        virtual_timer: unsafe { El1VirtualTimer::new() },
    };
    s32z2_main(peripherals);
    semihosting::process::exit(0);
}

/// The entry-point to the Rust application on Core 1
#[unsafe(no_mangle)]
pub extern "C" fn kmain_secondary() {
    unsafe extern "Rust" {
        safe fn s32z2_main2();
    }
    s32z2_main2();
    loop {
        aarch32_cpu::asm::wfe();
    }
}

/// If no main function is supplied for Core 1 in the application, we just sleep
#[unsafe(no_mangle)]
pub fn s32z2_main2_default() {
    loop {
        aarch32_cpu::asm::wfe();
    }
}

/// Get the Multi-Processor ID lowest byte (either 0 or 1 on this platform)
pub fn cpuid() -> u8 {
    aarch32_cpu::register::Mpidr::read().0 as u8
}

/// Setup RTU0 Core 0
fn setup_core() {
    // Enable the peripheral port in EL1
    let mut reg = aarch32_cpu::register::ImpPeriphpregionr::read();
    reg.0 |= 1;
    unsafe {
        aarch32_cpu::register::ImpPeriphpregionr::write(reg);
    }
    aarch32_cpu::asm::dsb();
    aarch32_cpu::asm::isb();
    // enable branch prediction, icache and dcache
    aarch32_cpu::register::Sctlr::modify(|w| {
        w.set_c(true);
        w.set_i(true);
        w.set_z(true);
    });
    aarch32_cpu::asm::dsb();
    aarch32_cpu::asm::isb();
    // Need the MPU be able to talk to the clock peripheral
    mpu::enable();
    // Turn on the PLLs
    clocks::configure_pll();

    // Wake up second core
    CORE1_RELEASED.store(true, core::sync::atomic::Ordering::Release);
    aarch32_cpu::asm::sev();
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _asm_secondary_core_park() {
    core::arch::naked_asm!(
        r#"
        // Some hardware register
        ldr     r0, ={core1_released}
    1:
        // Wait until Core 0 does a 'sev'
        wfe
        // Spin until register is non-zero.
        lda     r1, [r0]
        cmp     r1, 0
        beq     1b
        // return to start-up
        bx      lr
    "#,
    core1_released = sym CORE1_RELEASED
    );
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() {
    core::arch::naked_asm!(
        r#"
        mrc     p15, 0, r0, c0, c0, 5   // Read MPUID
        ands    r0, r0, 0xFF            // Is this Core 0?
        beq     1f
        b       _default_start          // Core 1 can leave right away
    1:
        // ECC init for S32Z2, which uses a table of (start, len)

        // r4 is the address of the current entry in the table.
        // Initialise it to the start of the table.
        ldr     r4, =__ecc_table_start__
        // skip to block loop termination check
        b       .Lecc_init_table_loop_check
        // process a table entry
    .Lecc_init_word_loop_start:
        // r5 counts how many 64-bit words have been written
        mov     r5, #0
        b       .Lecc_init_word_loop_check
    .Lecc_init_word_loop_inner:
        // load start address into r2 (length is in bytes)
        ldr     r2, [r4, #0]
        // multiply word count by 8 to get byte count
        lsls    r3, r5, #3
        // calculate address to write as (start + current index)
        adds    r1, r2, r3
        ldr     r2, =0x00000000
        ldr     r3, =0x00000000
        // write out one word to address in r1
        strd    r2, r3, [r1]
        // increment
        adds    r5, #1
        // load section length in bytes
        ldr     r3, [r4, #4]
        // divide length by eight
        lsrs    r2, r3, #3
        // compare counter with length
        cmp     r5, r2
    .Lecc_init_word_loop_check:
        // load the section length
        ldr     r3, [r4, #4]
        // divide section length by eight to get words
        lsrs    r2, r3, #3
        // have we written out enough words?
        cmp     r5, r2
        // if not, write some more words
        bcc     .Lecc_init_word_loop_inner
        // increment pointer to point at next table entry
        adds    r4, #8
    .Lecc_init_table_loop_check:
        // are we at the end of the table? (r4 points to the current table entry)
        ldr     r3, =__ecc_table_end__
        cmp     r4, r3
        // if not equal, go do some more
        bcc     .Lecc_init_word_loop_start

        /* TCM initialization */
        ldr     r0, =__TCMA_Start     /* Load new BASE address*/
        orr     r0, r0, #0x1b         /* 32k; EL0/1=ON L2=ON */
        mcr     p15, 0, r0, c9, c1, 0 /* Write to A-TCM config reg */

        ldr     r0, =__TCMB_Start     /* Load new BASE address */
        orr     r0, r0, #0x1b         /* 32k; EL0/1=ON L2=ON */
        mcr     p15, 0, r0, c9, c1, 1 /* Write to B-TCM config reg */

        ldr     r0, =__TCMC_Start     /* Load new BASE address*/
        orr     r0, r0, #0x1b         /* 32k; EL0/1=ON L2=ON */
        mcr     p15, 0, r0, c9, c1, 2 /* Write to C-TCM config reg */

        ldr     r0, =__TCMA_Start
        ldr     r1, =__TCMA_Length
        mov     r1, r1, lsr #5        /* Divide by 32 */
        bl      InitTcmLoop

        ldr     r0, =__TCMB_Start
        ldr     r1, =__TCMB_Length
        mov     r1, r1, lsr #5        /* Divide by 32 */
        bl      InitTcmLoop

        ldr     r0, =__TCMC_Start
        ldr     r1, =__TCMC_Length
        mov     r1, r1, lsr #5        /* Divide by 32 */
        bl      InitTcmLoop
       
        // Load Generic Timer frequency register before we leave EL2.
        // We're on a 40 MHz crystal and experimentally we have determined the
        // timer is running at 8 MHz, so there's probably a /5 divider. The
        // default value for CNTDIV is 4, and a zero divider doesn't make sense,
        // so that seems to stack up.
        ldr     r0, =8000000
        mcr     p15, 0, r0, c14, c0, 0

        // ECC init is now done
        b       _default_start

    InitTcmLoop:
        stm     r0, {{r4-r11}}        /* Move 8 location once 4*8=32 bytes */
        add     r0, r0, #32           /* Increment address by 32 */
        sub     r1 ,r1, #1            /* Decrement counter by 1 */
        cmp     r1, #0                /* Is the end of DMEM? */
        bne     InitTcmLoop           /* Restart loop if not */
        bx      lr
    "#
    )
}
