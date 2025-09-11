//! Common code for all S32Z2 examples

#![no_std]

// Need this to bring in the start-up function

use cortex_r_rt as _;
use panic_dcc as _;

mod clocks;
mod mpu;

/// The entry-point to the Rust application.
#[cortex_r_rt::entry]
fn kmain() -> ! {
    unsafe extern "Rust" {
        safe fn s32z2_main();
    }
    setup_core();
    s32z2_main();
    semihosting::process::exit(0);
}

/// Setup RTU0 Core 1
fn setup_core() {
    // Enable the peripheral port in EL1
    let mut reg = cortex_ar::register::ImpPeriphpregionr::read();
    reg.0 |= 1;
    unsafe {
        cortex_ar::register::ImpPeriphpregionr::write(reg);
    }
    cortex_ar::asm::dsb();
    cortex_ar::asm::isb();
    // enable branch prediction, icache and dcache
    cortex_ar::register::Sctlr::modify(|w| {
        w.set_c(true);
        w.set_i(true);
        w.set_z(true);
    });
    cortex_ar::asm::dsb();
    cortex_ar::asm::isb();
    // Need the MPU be able to talk to the clock peripheral
    mpu::enable();
    // Turn on the PLLs
    clocks::configure_pll();
}

// Custom start-up code for S32Z2
//
// Replaces the equivalent routine in cortex-r-rt, as we need to do extra things:
//
// * Erases the memory, so that we don't get ECC errors
// * Initialises the TCMs
// * Configures the Frequency register for the Generic Timer to 8 MHz
#[cfg(target_arch = "arm")]
core::arch::global_asm!(
    r#"
    .section .text.startup
    .align 0

    .global _start
    _start:
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
);
