/// HPET Timer implementation.
///
/// Adapted from https://sunriseos.github.io/SunriseOS/master/src/sunrise_kernel/devices/hpet.rs.html
use bitfield::*;
use core::fmt;
use core::fmt::Debug;
use core::fmt::Formatter;

use super::mmio::Mmio;

bitfield! {
    /// Represent the lower part of the General Capabilities and ID Register. [GCIDR]
    #[derive(Clone, Copy, Debug)]
    pub struct HpetIdRegister(u32);
    /// Indicates which revision of the function is implemented; must not be 0.
    pub revision_id, _ : 7, 0;
    /// The amount of timers - 1.
    pub timer_count_minus_one, _ : 12, 8;

    /// If this bit is 1, HPET main counter is capable of operating in 64 bit mode.
    pub counter_size_capability, _: 13;

    /// If this bit is 1, HPET is capable of using "legacy replacement" mapping.
    pub legacy_rt_capability, _: 15;

    /// Represent the HPET vendor id (most likely PCI vendor id?)
    pub vendor_id, _: 31, 16;
}

bitfield! {
    /// Represent the General Configuration Register. [GCR]
    #[derive(Clone, Copy, Debug)]
    pub struct HpetGeneralConfigurationRegister(u32);
    /// Control HPET activation (control main timer activation state and timer interrupts activation).
    pub enable_config, set_enable_config: 0;
    /// Control "legacy replacement" mapping activation state.
    pub legacy_rt_config, set_legacy_rt_config: 1;
}

bitfield! {
    /// Represent a Timer Configuration Register. [CCR]
    #[derive(Clone, Copy, Debug)]
    pub struct HpetTimerConfigurationRegister(u32);
    /// Control Timer Interrupt Type: 0 = Edge Trigger, 1 = Level Trigger
    pub interrupt_type, set_interrupt_type: 1;
    /// Control Timer Interrupt.
    pub interrupt_enable, set_interrupt_enable: 2;
    /// Control Timer Type: 0 = One Shot, 1 = Periodic
    pub timer_type, set_timer_type: 3;

    /// true if this timer is capable of periodic timer.
    pub periodic_interrupt_capability, _: 4;

    /// If this bit is 1, this timer is capable of operating in 64 bit mode.
    pub size_capability, _: 5;

    /// Set to 1 to allow software to write the accumulator data.
    ///
    /// # Note
    ///
    /// This auto-clear.
    pub accumulator_config, set_accumulator_config: 6;

    /// Set to 1 to force a 64 bit timer to operate as 32 bit one
    ///
    /// # Note
    ///
    /// This has no effect on a 32 bit timer.
    pub is_32bit_mode, set_32bit_mode: 8;

    /// Timer Interrupt Route: This indicate the routing in the I/O APIC
    ///
    /// # Note
    ///
    /// If the LegacyReplacement Route bit is set, then Timers 0 and 1 will have a different routing, and this bit field has no effect for those two timers.
    ///
    /// If the Timer FSB Interrupt bit is set, then the interrupt will be delivered directly to the FSB, and this bit field has no effect.
    pub interrupt_route, set_interrupt_route: 13, 9;

    /// Timer FSB Interrupt: force the interrupts to be delivered directly as FSB messages, rather than using the I/O APIC.
    pub fsb_interrupt, set_fsb_interrupt: 14;

    /// Timer FSB Interrupt Delivery capability.
    pub fsb_interrupt_capability, _: 15;
}

#[allow(clippy::missing_docs_in_private_items)]
#[repr(packed)]
/// Representation of HPET non variable registers.
pub struct HpetRegister {
    /// Information about the HPET model.
    pub identifier: Mmio<HpetIdRegister>, // 0x0
    /// Main counter tick period in femtoseconds (10^-15 seconds).
    /// Must not be zero, must be less or equal to 0x05F5E100, or 100 nanoseconds.
    pub period: Mmio<u32>, // 0x4
    _reserved0: u64, // 0x8
    /// General Configuration Register.
    pub general_configuration: Mmio<HpetGeneralConfigurationRegister>, // 0x10
    _reserved1: u32, // 0x14
    _reserved2: u64, // 0x18
    /// General Interrupt Status Register.
    pub general_interrupt_status: Mmio<u32>, // 0x20
    _reserved3: [u8; 0xCC], // 0x24
    /// main counter value.
    pub main_counter_value: Mmio<u64>, // 0xF0
    _reserved4: u64, // 0xF8
}

impl Debug for HpetRegister {
    /// Debug does not access reserved registers.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("HpetRegister")
            .field("identifier", &self.identifier)
            .field("period", &self.period)
            .field("general_configuration", &self.general_configuration)
            .field("general_interrupt_status", &self.general_interrupt_status)
            .field("main_counter_value", &self.main_counter_value)
            .finish()
    }
}
