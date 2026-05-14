//! Information about interrupts on this system

#[doc(hidden)]
pub use Interrupt as interrupt;

const INT_ID_SGI1: rtic::export::IntId = rtic::export::IntId::sgi(1);
const INT_ID_SGI2: rtic::export::IntId = rtic::export::IntId::sgi(2);
const INT_ID_VIRT_TIMER: rtic::export::IntId = rtic::export::IntId::ppi(11);
const INT_ID_PHYS_TIMER: rtic::export::IntId = rtic::export::IntId::ppi(14);

/// Interrupts we want RTIC to be able to use
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Interrupt {
    Sgi1,
    Sgi2,
    VirtTimer,
    PhysTimer,
}

impl Interrupt {
    pub const fn as_intid(self) -> rtic::export::IntId {
        match self {
            Self::Sgi1 => INT_ID_SGI1,
            Self::Sgi2 => INT_ID_SGI2,
            Self::VirtTimer => INT_ID_VIRT_TIMER,
            Self::PhysTimer => INT_ID_PHYS_TIMER,
        }
    }
}

impl From<Interrupt> for rtic::export::IntId {
    fn from(value: Interrupt) -> Self {
        value.as_intid()
    }
}

impl From<rtic::export::IntId> for Interrupt {
    fn from(value: rtic::export::IntId) -> Self {
        if value == INT_ID_SGI1 {
            Interrupt::Sgi1
        } else if value == INT_ID_SGI2 {
            Interrupt::Sgi2
        } else if value == INT_ID_VIRT_TIMER {
            Interrupt::VirtTimer
        } else if value == INT_ID_PHYS_TIMER {
            Interrupt::PhysTimer
        } else {
            panic!("Unknown interrupt ID {:?}", value);
        }
    }
}

impl PartialEq<rtic::export::IntId> for Interrupt {
    fn eq(&self, other: &rtic::export::IntId) -> bool {
        let self_int_id: rtic::export::IntId = (*self).into();
        self_int_id == *other
    }
}
