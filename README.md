# Rust Demo for the NXP32Z270-400EVB

This is a demonstration of bare-metal Rust code running on the NXP S32E2/S32Z2
platform.

The demo is built for the [NXP S32Z280-400EVB] evaluation board, which
features an [NXP S32Z280 SoC]. This SoC includes:

[NXP S32Z280-400EVB]: https://www.nxp.com/part/S32Z280-400EVB
[NXP S32Z280 SoC]: https://www.nxp.com/docs/en/data-sheet/S32E27.pdf

* Eight Arm Cortex-R52 cores, as two clusters of two lockstep pairs
* 16MB of RAM (8MB per cluster)
* Two Arm Cortex-M33 system manager cores as one lockstep pair

This demo runs from RAM on the first Cortex-R52 lockstep pair in the first
Cluster. It initialises the MMU, checks the PLL configuration, and prints to a
debug console inside the TRACE32 IDE using the Arm DCC protocol.

## Requirements

* Ferrocene
* CriticalUp
* Lauterbach's TRACE32 for Arm
* Lauterbach PowerView X50
* The NXP S32Z280-400EVB Board

You also be able to load the firmware into RAM using NXP S32 Design Studio IDE, or any other JTAG programmer compatible with the S32E2/S32Z2 platform.

## Compilation

To compile the examples, run:

```bash
criticalup install
critical link create
cargo build
```

## Debugging

To load and debug the examples, execute the
[`./t32-scripts/start-s32z270dc-rtu0.cmm`](./t32-scripts/start-s32z270dc-rtu0.cmm)
in TRACE32 for Arm. You can modify the script to select which binary to load
and run.

## Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on Ferrocene 25.05 and up. It *might*
compile with older versions but that may change in any new patch release.

It uses the `armv8r-none-eabihf` target which is available in Ferrocene through
criticalup. To use upstream Rust, you need the nightly toolchain.

## Licence

Copyright (c) Ferrous Systems, 2025

Licensed under either [MIT](./LICENSE-MIT) or [Apache-2.0](./LICENSE-APACHE) at
your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be licensed as above, without any
additional terms or conditions.
