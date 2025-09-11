//! Clock configuration code for the S32Z2
//!
//! Programs the *DFS* (Digital Frequency Synthesizer).

use arbitrary_int::{u15, u3, u6};
use arm_dcc::dprintln as println;

/// The DFS Peripheral
#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct Dfs {
    _reserved: [u32; 3],
    portsr: DfsPortSr,
    portlolsr: u32,
    portreset: DfsPortReset,
    ctl: DfsCtl,
    dvports: [DfsDvPort; 6],
}

/// The DFS Port Status Register
#[bitbybit::bitfield(u32)]
pub struct DfsPortSr {
    /// Lock Status for Port 5
    #[bit(5, r)]
    p5_locked: bool,
    /// Lock Status for Port 4
    #[bit(4, r)]
    p4_locked: bool,
    /// Lock Status for Port 3
    #[bit(3, r)]
    p3_locked: bool,
    /// Lock Status for Port 2
    #[bit(2, r)]
    p2_locked: bool,
    /// Lock Status for Port 1
    #[bit(1, r)]
    p1_locked: bool,
    /// Lock Status for Port 0
    #[bit(0, r)]
    p0_locked: bool,
}

impl core::fmt::Debug for DfsPortSr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DfsPortSr({:06b})", self.raw_value())
    }
}

/// The DFS Control Register
#[bitbybit::bitfield(u32)]
pub struct DfsCtl {
    /// If true, the DFS phase generator is in reset and you cannot enable any ports
    #[bit(1, rw)]
    in_reset: bool,
}

impl core::fmt::Debug for DfsCtl {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DfsCtl(in_reset={})", self.in_reset())
    }
}

/// The DFS Port Reset Register
#[bitbybit::bitfield(u32)]
pub struct DfsPortReset {
    /// If true, Port 5 is disabled
    #[bit(5, rw)]
    reset5: bool,
    /// If true, Port 4 is disabled
    #[bit(4, rw)]
    reset4: bool,
    /// If true, Port 3 is disabled
    #[bit(3, rw)]
    reset3: bool,
    /// If true, Port 2 is disabled
    #[bit(2, rw)]
    reset2: bool,
    /// If true, Port 1 is disabled
    #[bit(1, rw)]
    reset1: bool,
    /// If true, Port 0 is disabled
    #[bit(0, rw)]
    reset0: bool,
}

impl core::fmt::Debug for DfsPortReset {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DfsPortReset({:06b})", self.raw_value())
    }
}

/// Divider configuration for DFS output port
#[bitbybit::bitfield(u32)]
pub struct DfsDvPort {
    /// Integer part of division value
    #[bits(8..=15, rw)]
    mfi: u8,
    /// Numerator of fractional part of division value
    #[bits(0..=5, rw)]
    mfn: u6,
}

impl core::fmt::Debug for DfsDvPort {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DfsDvPort(mfi={}, mfn={})", self.mfi(), self.mfn())
    }
}

/// The PLL Digital Interface Peripheral
#[derive(derive_mmio::Mmio)]
#[repr(C)]
pub struct PllDig {
    /// PLL Control, offset: 0x0
    pllcr: PllDigCr,
    /// PLL Status, offset: 0x4
    pllsr: PllDigSr,
    /// PLL Divider, offset: 0x8
    plldv: PllDigDv,
    /// PLL Frequency Modulation, offset: 0xC, available only on: CORE_PLL, DDR_PLL (missing on PERIPH_PLL)
    pllfm: u32,
    /// PLL Fractional Divider, offset: 0x10
    pllfd: PllDigFd,
    _reserved: [u32; 3],
    /// PLL Clock Multiplexer, offset: 0x20
    pllclkmux: PllDigClkMux,
    _reserved1: [u32; 23],
    /// PLL Dividers
    pllodiv: [u32; 6],
}

/// The PLL Status Register
#[bitbybit::bitfield(u32)]
pub struct PllDigSr {
    /// If true, Loss of Lock detected
    #[bit(3, rw)]
    lol: bool,
    /// If true, PLL is locked
    #[bit(2, rw)]
    locked: bool,
}

impl core::fmt::Debug for PllDigSr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PllDigSr(lol={}, locked={})", self.lol(), self.locked())
    }
}

/// The PLL Control Register
#[bitbybit::bitfield(u32)]
pub struct PllDigCr {
    /// If 1, the PLL is currently in the power-down state
    #[bit(31, rw)]
    pd: bool,
}

impl core::fmt::Debug for PllDigCr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PllDigCr(pd={})", self.pd())
    }
}

/// The PLL Divider Register
#[bitbybit::bitfield(u32)]
pub struct PllDigDv {
    #[bits(12..=14, rw)]
    rdiv: u3,
    #[bits(0..=7, rw)]
    mfi: u8,
}

impl core::fmt::Debug for PllDigDv {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PllDigDv(rdiv={}, mfi={})", self.rdiv(), self.mfi())
    }
}

/// The PLL Fractional Divider Register
#[bitbybit::bitfield(u32)]
pub struct PllDigFd {
    #[bit(30, rw)]
    sdmen: bool,
    #[bits(0..=14, rw)]
    mfn: u15,
}

impl core::fmt::Debug for PllDigFd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PllDigFd(sdmen={}, mfn={})", self.sdmen(), self.mfn())
    }
}

/// The PLL Clock Mux
#[bitbybit::bitfield(u32)]
pub struct PllDigClkMux {
    /// If true, select external Fast Crystal Oscillator (FXOSC).
    ///
    /// Otherwise, select Fast Internal RC (FIRC)
    #[bit(0, rw)]
    select_fxosc: bool,
}

impl core::fmt::Debug for PllDigClkMux {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "PllDigClkMux({})",
            if self.select_fxosc() { "FXOSC" } else { "FIRC" }
        )
    }
}

/// Configure the PLLs on the S32Z2
///
/// Acutally it seems the PLLs are already running, so this actually
/// just prints the configuration.
pub fn configure_pll() {
    let mut ip_core_dfs = unsafe { Dfs::new_mmio_at(0x4026_0000) };
    let mut ip_periph_dfs = unsafe { Dfs::new_mmio_at(0x4027_0000) };
    let mut ip_core_pll = unsafe { PllDig::new_mmio_at(0x4021_0000) };
    let mut ip_periph_pll = unsafe { PllDig::new_mmio_at(0x4022_0000) };

    print_clock_setup("core", &mut ip_core_dfs, &mut ip_core_pll);
    print_clock_setup("periph", &mut ip_periph_dfs, &mut ip_periph_pll);
}

fn print_clock_setup(name: &str, dfs: &mut MmioDfs, pll: &mut MmioPllDig) {
    // The clocks seem to be set up for us
    println!("Examining {} DFS and PLL...", name);
    println!("  {:#?}", dfs.read_ctl());
    println!("  {:#?}", dfs.read_portsr());
    println!("  {:#?}", dfs.read_portreset());
    for i in 0.. {
        if let Ok(p) = dfs.read_dvports(i) {
            println!("  - DvPort{}: {:#?}", i, p);
        } else {
            break;
        }
    }
    println!("  {:#?}", pll.read_pllcr());
    println!("  {:#?}", pll.read_pllsr());
    println!("  {:#?}", pll.read_plldv());
    println!("  {:#?}", pll.read_pllfd());
}
