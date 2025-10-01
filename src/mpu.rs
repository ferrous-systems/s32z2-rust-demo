//! S32Z2 MPU set-up code
//!
//! The configuration of the MPU on this chip is *mandatory*. You cannot access
//! any peripherals using the default MPU 'background' configuration that
//! applies when the MPU is disabled.

use cortex_ar::{
    self as _,
    pmsav8::{
        Cacheable, El1AccessPerms, El1Config, El1Mpu, El1Region, El1Shareability, MemAttr,
        RwAllocPolicy,
    },
};

/// Enable extra debug output over DCC
static VERBOSE_DEBUGGING: bool = false;

/// Index of MAIR Attr used for code regions
const MPU_MAIR_INDEX_CODE: u8 = 0;

/// Index of MAIR Attr used for data regions
const MPU_MAIR_INDEX_DATA: u8 = 1;

/// Index of MAIR Attr used for peripheral regions
const MPU_MAIR_INDEX_DEVICE: u8 = 2;

/// Basic MPU config for the S32Z2
static MPU_CONFIG: El1Config = El1Config {
    background_config: false,
    regions: &[
        // Code in R52_0_0_CODE_RAM
        El1Region {
            range: 0x3210_0000 as *mut u8..=0x321B_FFFF as *mut u8,
            shareability: El1Shareability::InnerShareable,
            // ordinarily you'd want this read-only, except the debugger
            // replaces instructions on-the-fly with soft breakpoints, so
            // it has to be read-write if you want single-step debugging to work.
            access: El1AccessPerms::ReadWrite,
            no_exec: false,
            mair: MPU_MAIR_INDEX_CODE,
            enable: true,
        },
        // Data in R52_0_0_CODE_RAM
        El1Region {
            range: 0x3178_0000 as *mut u8..=0x317B_FFFF as *mut u8,
            shareability: El1Shareability::InnerShareable,
            access: El1AccessPerms::ReadWrite,
            no_exec: true,
            mair: MPU_MAIR_INDEX_DATA,
            enable: true,
        },
        // RTU0 P0 Peripherals
        El1Region {
            range: 0x4000_0000 as *mut u8..=0x407F_FFFF as *mut u8,
            shareability: El1Shareability::NonShareable,
            access: El1AccessPerms::ReadWriteNoEL0,
            no_exec: true,
            mair: MPU_MAIR_INDEX_DEVICE,
            enable: true,
        },
        // RTU0 GICv3
        El1Region {
            range: 0x4780_0000 as *mut u8..=0x479F_FFFF as *mut u8,
            shareability: El1Shareability::NonShareable,
            access: El1AccessPerms::ReadWriteNoEL0,
            no_exec: true,
            mair: MPU_MAIR_INDEX_DEVICE,
            enable: true,
        },
    ],
    memory_attributes: &[
        // MPU_MAIR_INDEX_CODE
        MemAttr::NormalMemory {
            outer: Cacheable::WriteThroughNonTransient(RwAllocPolicy::R),
            inner: Cacheable::WriteThroughNonTransient(RwAllocPolicy::R),
        },
        // MPU_MAIR_INDEX_DATA
        MemAttr::NormalMemory {
            outer: Cacheable::WriteBackNonTransient(RwAllocPolicy::R),
            inner: Cacheable::WriteBackNonTransient(RwAllocPolicy::R),
        },
        // MPU_MAIR_INDEX_DEVICE
        MemAttr::DeviceMemory,
    ],
};

/// Set up the MPU
///
/// This is *mandatory* on S32Z2 because the peripherals are in
/// 'Normal' memory according to the default MPU memory map, which
/// absolutely does not work for talking to peripherals.
pub fn enable() {
    let mut mpu = unsafe { El1Mpu::new() };
    if VERBOSE_DEBUGGING {
        arm_dcc::dprintln!("MPU Config before:");
        for idx in 0..mpu.num_regions() {
            if let Some(region) = mpu.get_region(idx) {
                if region.enable {
                    arm_dcc::dprintln!("{:02}: {:?}", idx, region);
                }
            }
        }
    }

    mpu.configure(&MPU_CONFIG).expect("MPU Config");

    arm_dcc::dprintln!("MPU Config after:");
    for idx in 0..mpu.num_regions() {
        if let Some(region) = mpu.get_region(idx) {
            if region.enable {
                arm_dcc::dprintln!("{:02}: {:?}", idx, region);
            }
        }
    }
    mpu.enable();
}
