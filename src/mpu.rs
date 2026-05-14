//! S32Z2 MPU set-up code
//!
//! The configuration of the MPU on this chip is *mandatory*. You cannot access
//! any peripherals using the default MPU 'background' configuration that
//! applies when the MPU is disabled.

use aarch32_cpu::{
    self as _,
    pmsav8::{
        CachePolicy, El1AccessPerms, El1Config, El1Region, El1Shareability, MemAttr, RwAllocPolicy,
    },
};

/// Index of MAIR Attr used for code regions
const MPU_MAIR_INDEX_CODE: u8 = 0;

/// Index of MAIR Attr used for data regions
const MPU_MAIR_INDEX_DATA: u8 = 1;

/// Index of MAIR Attr used for peripheral regions
const MPU_MAIR_INDEX_DEVICE: u8 = 2;

/// Basic MPU config for the S32Z2
pub static MPU_CONFIG: El1Config = El1Config {
    background_config: false,
    regions: &[
        // Code in R52_0_0_CODE_RAM
        El1Region {
            range: 0x3210_0000 as *const u8..=0x321B_FFFF as *const u8,
            shareability: El1Shareability::InnerShareable,
            // ordinarily you'd want this read-only, except the debugger
            // replaces instructions on-the-fly with soft breakpoints, so
            // it has to be read-write if you want single-step debugging to work.
            access: El1AccessPerms::ReadWrite,
            no_exec: false,
            mair: MPU_MAIR_INDEX_CODE,
            enable: true,
        },
        // Data in R52_0_0_DATA_RAM
        El1Region {
            range: 0x3178_0000 as *const u8..=0x317B_FFFF as *const u8,
            shareability: El1Shareability::InnerShareable,
            access: El1AccessPerms::ReadWrite,
            no_exec: true,
            mair: MPU_MAIR_INDEX_DATA,
            enable: true,
        },
        // RTU0 P0 Peripherals
        El1Region {
            range: 0x4000_0000 as *const u8..=0x407F_FFFF as *const u8,
            shareability: El1Shareability::NonShareable,
            access: El1AccessPerms::ReadWriteNoEL0,
            no_exec: true,
            mair: MPU_MAIR_INDEX_DEVICE,
            enable: true,
        },
        // RTU0 GICv3
        El1Region {
            range: 0x4780_0000 as *const u8..=0x479F_FFFF as *const u8,
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
            outer: CachePolicy::WriteThroughNonTransient(RwAllocPolicy::R),
            inner: CachePolicy::WriteThroughNonTransient(RwAllocPolicy::R),
        },
        // MPU_MAIR_INDEX_DATA
        MemAttr::NormalMemory {
            outer: CachePolicy::WriteBackNonTransient(RwAllocPolicy::R),
            inner: CachePolicy::WriteBackNonTransient(RwAllocPolicy::R),
        },
        // MPU_MAIR_INDEX_DEVICE
        MemAttr::DeviceMemory,
    ],
};
