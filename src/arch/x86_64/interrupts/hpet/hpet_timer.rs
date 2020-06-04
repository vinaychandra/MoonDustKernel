use super::mmio::*;
use super::registers::*;

#[derive(Debug)]
/// Representation of an HPET timer registers.
pub struct HpetTimerRegister {
    /// The configuration and capabilities register of this timer.
    pub config: Mmio<HpetTimerConfigurationRegister>,
    /// Routing capability (IRQ0 to IRQ31 on the I/O APIC).
    pub interrupt_route_capability: Mmio<u32>,
    /// The comparator value register low part.
    pub comparator_value_low: Mmio<u32>,
    /// The comparator value register high part.
    pub comparator_value_high: Mmio<u32>,
    /// The FSB Interrupt Route register lower part (value during FSB interrupt message).
    pub fsb_value: Mmio<u32>,
    /// The FSB Interrupt Route register higher part (address used during FSB interrupt message).
    pub fsb_address: Mmio<u32>,
}

#[derive(Debug)]
/// Represent an HPET timer.
pub struct HpetTimer {
    /// The mmio address of this HPET timer.
    inner: *mut HpetTimerRegister,

    /// Cached value of ``HpetTimerConfigurationRegister::size_capability``.
    support_64bit: bool,

    /// Cached value of ``HpetTimerConfigurationRegister::periodic_interrupt_capability``.
    support_periodic_interrupt: bool,

    /// Cached value of ``HpetTimerConfigurationRegister::fsb_interrupt_capability``.
    support_fsb_interrupt: bool,

    /// Cached value of ``HpetTimerRegister::interrupt_route_capability``.
    interrupt_route_capability: u32,
}

impl HpetTimer {
    /// This is the maximum IRQ lines supported by the HPET.
    const MAX_IRQ: u32 = 0x1F;

    /// Create a new HPET timer instance from MMIO registers.
    pub fn new(inner: *mut HpetTimerRegister) -> Self {
        let mut res = HpetTimer {
            inner,
            support_64bit: false,
            support_periodic_interrupt: false,
            support_fsb_interrupt: false,
            interrupt_route_capability: 0,
        };

        let config = res.get_inner().config.read();

        res.support_64bit = config.size_capability();
        res.support_periodic_interrupt = config.periodic_interrupt_capability();
        res.support_fsb_interrupt = config.fsb_interrupt_capability();
        res.interrupt_route_capability = res.get_inner().interrupt_route_capability.read();
        res
    }

    /// Return true if this timer is a 64 bits timer.
    pub fn support_64bit(&self) -> bool {
        self.support_64bit
    }

    /// Return true if this timer supports periodic interrupt.
    pub fn support_periodic_interrupt(&self) -> bool {
        self.support_periodic_interrupt
    }

    /// Return true if this timer supports fsb interrupt.
    pub fn support_fsb_interrupt(&self) -> bool {
        self.support_fsb_interrupt
    }

    /// Return true if the timer support routing to the given IRQ.
    pub fn support_interrupt_routing(&self, index: u32) -> bool {
        if index > Self::MAX_IRQ {
            return false;
        }

        let irq_mask = 1 << index;
        (self.interrupt_route_capability & irq_mask) == irq_mask
    }

    /// Set the routing for the interrupt to the I/O APIC.
    ///
    /// # Panics
    ///
    /// Panics if the given interrupt route is not supported by this hpet timer.
    pub fn set_interrupt_route(&self, index: u32) {
        let mut config = self.get_inner().config.read();
        config.set_interrupt_route(index);
        self.get_inner().config.write(config);

        let config = self.get_inner().config.read();
        assert!(
            config.interrupt_route() == index,
            "Illegal interrupt route (as tested). Supported routes: {}.",
            self.interrupt_route_capability
        );
    }

    /// Set the timer comparactor value
    pub fn set_comparator_value(&self, value: u64) {
        self.get_inner()
            .comparator_value_low
            .write((value & 0xFFFF_FFFF) as u32);
        self.get_inner()
            .comparator_value_high
            .write((value >> 32) as u32);
    }

    /// Set the timer accumulator value.
    ///
    /// # Note
    ///
    /// The timer MUST be in periodic mode.
    pub fn set_accumulator_value(&self, value: u64) {
        // We update the accumulator register two times.
        // TODO: Test the hardware behaviour on partial write of the HPET accumulator
        // BODY: Because we are running on i386, this cause issue on QEMU.
        // BODY: In fact, QEMU clear the accumulator flag on every partial write.
        // BODY: The question here is: Is that normal or a bug in QEMU?
        let mut config = self.get_inner().config.read();
        config.set_accumulator_config(true);
        self.get_inner().config.write(config);
        self.get_inner()
            .comparator_value_low
            .write((value & 0xFFFF_FFFF) as u32);

        let mut config = self.get_inner().config.read();
        config.set_accumulator_config(true);
        self.get_inner().config.write(config);
        self.get_inner()
            .comparator_value_high
            .write((value >> 32) as u32);
    }

    /// Set Edge Trigger.
    pub fn set_edge_trigger(&self) {
        let mut config = self.get_inner().config.read();
        config.set_interrupt_type(false);
        self.get_inner().config.write(config);
    }

    /// Set Level Trigger.
    pub fn set_level_trigger(&self) {
        let mut config = self.get_inner().config.read();
        config.set_interrupt_type(true);
        self.get_inner().config.write(config);
    }

    /// Set the timer in One Shot mode.
    pub fn set_one_shot_mode(&self) {
        let mut config = self.get_inner().config.read();
        config.set_timer_type(false);
        self.get_inner().config.write(config);
    }

    /// Set the timer in Periodic mode.
    ///
    /// # Note
    ///
    /// The timer must support periodic mode.
    pub fn set_periodic_mode(&self) {
        let mut config = self.get_inner().config.read();
        config.set_timer_type(true);
        self.get_inner().config.write(config);
    }

    /// Enable interrupt.
    pub fn enable_interrupt(&self) {
        let mut config = self.get_inner().config.read();
        config.set_interrupt_enable(true);
        self.get_inner().config.write(config);
    }

    /// Disable interrupt.
    pub fn disable_interrupt(&self) {
        let mut config = self.get_inner().config.read();
        config.set_interrupt_enable(false);
        self.get_inner().config.write(config);
    }

    /// Determine if the interrupt is enabled.
    pub fn has_interrupt_enabled(&self) -> bool {
        self.get_inner().config.read().interrupt_enable()
    }

    fn get_inner(&self) -> &mut HpetTimerRegister {
        unsafe { &mut (*self.inner) }
    }
}
