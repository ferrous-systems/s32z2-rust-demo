# Rust Demo for the NXP32Z270-400EVB

## Requirements

* Ferrocene
* CriticalUp
* Lauterbach's TRACE32 for Arm
* Lauterbach PowerView X50
* The NXPS32Z70-400EVB Board

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
in TRACE32 for Arm.

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
