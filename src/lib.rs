//! Common code for all S32Z2 examples

#![no_std]

use core::{ptr::NonNull, sync::atomic::AtomicBool};

use aarch32_cpu::generic_timer::El1VirtualTimer;
#[cfg(target_arch = "arm")]
use aarch32_cpu::register::{cpsr::ProcessorMode, Cpsr, Hactlr};
use aarch32_rt as _;
use arm_dcc::dprintln as println;
use arm_gic::{
    gicv3::{GicCpuInterface, GicV3},
    UniqueMmioPointer,
};
use panic_dcc as _;

mod clocks;
mod mpu;

/// Offset from PERIPHBASE for GIC Distributor
pub const GICD_BASE_OFFSET: usize = 0x0000_0000usize;

/// Offset from PERIPHBASE for the first GIC Redistributor
pub const GICR_BASE_OFFSET: usize = 0x0010_0000usize;

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
pub extern "C" fn kmain2() {
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
    CORE1_RELEASED.store(true, core::sync::atomic::Ordering::Relaxed);
    aarch32_cpu::asm::sev();
}

// Custom start-up code for S32Z2
//
// Supplements the equivalent routine in aarch32-rt, as we need to do extra things:
//
// * Erase the memory, so that we don't get ECC errors
// * Initialise the TCMs
// * Configure the Frequency register for the Generic Timer to 8 MHz
// * Set-up Core 1 with its own stacks, and set it running kmain2 at EL1
#[cfg(target_arch = "arm")]
core::arch::global_asm!(
    r#"
    .section .text.startup
    .align 0

    .global _start
    .type _start,%function
    _start:
        // Read MPIDR into R0
        mrc     p15, 0, r0, c0, c0, 5
        ands    r0, r0, 0xFF
        beq     core0_init
    core1_init:
        ldr     r0, ={released_bool}
        mov     r1, #0
    core1_spin:
        wfe
        // spin until boolean is set.
        ldr     r2, [r0]  
        cmp     r1, r2
        beq     core1_spin
    core1_released:
        // First we must exit EL2...
        // Set the HVBAR (for EL2) to _vector_table
        ldr     r0, =_vector_table
        mcr     p15, 4, r0, c12, c0, 0
        // Configure HACTLR to let us enter EL1
        mrc     p15, 4, r0, c1, c0, 1
        mov     r1, {hactlr_bits}
        orr     r0, r0, r1
        mcr     p15, 4, r0, c1, c0, 1
        // Program the SPSR - enter system mode (0x1F) in Arm mode with IRQ, FIQ masked
        mov		r0, {sys_mode}
        msr		spsr_hyp, r0
        adr		r0, start_core1_el1
        msr		elr_hyp, r0
        dsb
        isb
        eret
    start_core1_el1:
        // Allow VFP coprocessor access
        mrc     p15, 0, r0, c1, c0, 2
        orr     r0, r0, #0xF00000
        mcr     p15, 0, r0, c1, c0, 2
        // Enable VFP
        mov     r0, #0x40000000
        vmsr    fpexc, r0
        // Set the VBAR (for EL1) to _vector_table.
        ldr     r0, =_vector_table
        mcr     p15, 0, r0, c12, c0, 0
        // set up our stacks - also switches to SYS mode
        movs    r0, #1
        bl      _stack_setup_preallocated
        // Zero all registers before calling kmain2
        mov     r0, 0
        mov     r1, 0
        mov     r2, 0
        mov     r3, 0
        mov     r4, 0
        mov     r5, 0
        mov     r6, 0
        mov     r7, 0
        mov     r8, 0
        mov     r9, 0
        mov     r10, 0
        mov     r11, 0
        mov     r12, 0
        // call our kmain2 for core 1
        bl      kmain2
    core0_init:
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
    "#,
    released_bool = sym CORE1_RELEASED,
    hactlr_bits = const {
        Hactlr::new_with_raw_value(0)
            .with_cpuactlr(true)
            .with_cdbgdci(true)
            .with_flashifregionr(true)
            .with_periphpregionr(true)
            .with_qosr(true)
            .with_bustimeoutr(true)
            .with_intmonr(true)
            .with_err(true)
            .with_testr1(true)
            .raw_value()
    },
    sys_mode = const {
        Cpsr::new_with_raw_value(0)
            .with_mode(ProcessorMode::Sys)
            .with_i(true)
            .with_f(true)
            .raw_value()
    },
);
