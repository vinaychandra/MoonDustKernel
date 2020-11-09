use super::hpet_timer::*;
use super::registers::*;

#[derive(Debug)]
/// Represent an HPET device.
pub struct Hpet {
    /// The mmio address of this HPET device.
    pub inner: *mut HpetRegister,

    /// Cached value of ``Hpet::period`` in femtoseconds.
    period: u32,

    /// The count of timer of this HPET device.
    timer_count: u32,
}

impl Hpet {
    /// Create a new HPET device instance from MMIO registers.
    pub fn new(inner: *mut HpetRegister) -> Self {
        let mut res = Hpet {
            inner,
            timer_count: 1,
            period: 0,
        };
        res.timer_count = unsafe { (*res.inner).identifier.read().timer_count_minus_one() } + 1;
        res.period = unsafe { (*res.inner).period.read() };

        res
    }

    /// Return true if the device supports "legacy mapping".
    pub fn has_legacy_mapping(&self) -> bool {
        unsafe { (*self.inner).identifier.read().legacy_rt_capability() }
    }

    /// Return the period of the HPET device.
    pub fn get_period(&self) -> u32 {
        self.period
    }

    /// Return the frequency of the HPET device.
    pub fn get_frequency(&self) -> u64 {
        1_000_000_000_000_000 / u64::from(self.get_period())
    }

    /// Enable the "legacy mapping".
    pub fn enable_legacy_mapping(&self) {
        let mut general_configuration = unsafe { (*self.inner).general_configuration.read() };
        general_configuration.set_legacy_rt_config(true);
        unsafe {
            (*self.inner)
                .general_configuration
                .write(general_configuration)
        }
    }

    /// Disable the "legacy mapping".
    pub fn disable_legacy_mapping(&self) {
        let mut general_configuration = unsafe { (*self.inner).general_configuration.read() };
        general_configuration.set_legacy_rt_config(false);
        unsafe {
            (*self.inner)
                .general_configuration
                .write(general_configuration)
        }
    }

    /// Check "legacy mapping" status.
    pub fn is_legacy_mapping_enabled(&self) -> bool {
        let general_configuration = unsafe { (*self.inner).general_configuration.read() };
        general_configuration.legacy_rt_config()
    }

    /// Enable HPET (main timer running, and timer interrupts allowed).
    pub fn enable(&self) {
        let mut general_configuration = unsafe { (*self.inner).general_configuration.read() };
        general_configuration.set_enable_config(true);
        unsafe {
            (*self.inner)
                .general_configuration
                .write(general_configuration)
        }
    }

    /// Set HPET main counter value.
    pub fn set_main_counter_value(&self, value: u64) {
        unsafe { (*self.inner).main_counter_value.write(value) }
    }

    /// Get HPET main counter value.
    pub fn get_main_counter_value(&self) -> u64 {
        unsafe { (*self.inner).main_counter_value.read() }
    }

    /// Disable HPET (main timer halted, and timer interrupts disabled).
    pub fn disable(&self) {
        let mut general_configuration = unsafe { (*self.inner).general_configuration.read() };
        general_configuration.set_enable_config(false);
        unsafe {
            (*self.inner)
                .general_configuration
                .write(general_configuration)
        }
    }

    /// Check HPET status.
    pub fn is_enabled(&self) -> bool {
        let general_configuration = unsafe { (*self.inner).general_configuration.read() };
        general_configuration.enable_config()
    }

    /// Get a timer at the given index.
    pub fn get_timer(&self, index: u32) -> Option<HpetTimer> {
        if index >= self.timer_count {
            return None;
        }
        let mmio_base_address = self.inner as usize;

        let timer_address = mmio_base_address + 0x100 + (0x20 * index) as usize;
        Some(HpetTimer::new(timer_address as *mut HpetTimerRegister))
    }
}

unsafe impl Sync for Hpet {}
