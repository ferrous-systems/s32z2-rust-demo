MEMORY {
    R52_0_0_TCMA (rw)       : ORIGIN = 0x30000000, LENGTH = 0x10000
    R52_0_0_TCMB (rw)       : ORIGIN = 0x30100000, LENGTH = 0x4000
    R52_0_0_TCMC (rw)       : ORIGIN = 0x30200000, LENGTH = 0x4000
    R52_0_0_CODE_RAM (rx)   : ORIGIN = 0x32100000, LENGTH = 0x1C0000
    R52_0_0_DATA_RAM (rw)   : ORIGIN = 0x31780000, LENGTH = 0x40000
}

SECTIONS {
    /*
     * ECC initialization is done by 64-bit writes thus the pointers and lengths
     * found within this table must be 8-byte aligned. The table itself only
     * needs to be 4-byte aligned.
     */
    .ecc.table :
    {
        . = ALIGN(4);
        __ecc_table_start__ = .;

        /* Erase all of R52_0_0_DATA_RAM - these values are a multiple of 8 */
        LONG (ORIGIN(R52_0_0_DATA_RAM))
        LONG (LENGTH(R52_0_0_DATA_RAM))

        __ecc_table_end__ = .;
    } > R52_0_0_CODE_RAM
} INSERT AFTER .text;

REGION_ALIAS("VECTORS", R52_0_0_CODE_RAM);
REGION_ALIAS("CODE", R52_0_0_CODE_RAM);
REGION_ALIAS("DATA", R52_0_0_DATA_RAM);

__TCMA_Start  = ORIGIN(R52_0_0_TCMA);
__TCMA_Length = LENGTH(R52_0_0_TCMA);
__TCMB_Start  = ORIGIN(R52_0_0_TCMB);
__TCMB_Length = LENGTH(R52_0_0_TCMB);
__TCMC_Start  = ORIGIN(R52_0_0_TCMC);
__TCMC_Length = LENGTH(R52_0_0_TCMC);
