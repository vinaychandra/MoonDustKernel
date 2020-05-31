use core::convert::TryInto;
use core::ptr;
use x86_64::VirtAddr;

/// HPET Timer.
pub struct HPET {
    /// The base address for HPET.
    /// HPET register i is at address `base_address` + i*8.
    base_address: VirtAddr,

    /// The minimum timer length for HPET to properly
    /// fire interrupts.
    minimum_timer_length: u64,

    /// Is 64 bit counter available?
    long_counter_available: bool,

    /// Number of available counters.
    number_of_counters: u8,

    /// Period between ticks in femptoseconds.
    period_between_ticks: u64,
}

const GCIDR: u64 = 0x00;
const GCR: u64 = 0x02;
const MCR: u64 = 0x1e;
const CCRB: u64 = 0x20;
const CVRB: u64 = 0x21;

impl HPET {
    /// Create the instance of HPET.
    /// The physical base address can be found in the ACPI tables.
    pub fn new(base_address: VirtAddr, minimum_timer_length: u64) -> HPET {
        let gcidr = HPET::hpet_read(base_address, GCIDR);

        // Is Long counter supported? bit index 13
        let long_counter = gcidr & (1 << 13) > 0;

        // Number of counters available: bits 8:12
        let max_counter_index = (gcidr & 0b1_1111_0000_0000) >> 8;
        let number_of_counters = max_counter_index + 1;

        // Period between ticks in femto seconds: bits 32:63
        let period_between_ticks = gcidr >> 32;

        HPET {
            base_address,
            minimum_timer_length,
            long_counter_available: long_counter,
            number_of_counters: number_of_counters.try_into().unwrap(),
            period_between_ticks,
        }
    }

    pub fn one_shot_timer(&self, mut number_of_ticks_in_future: u64) {
        if number_of_ticks_in_future < self.minimum_timer_length {
            number_of_ticks_in_future = self.minimum_timer_length;
        }
    }

    fn hpet_read(base_address: VirtAddr, index: u64) -> u64 {
        let virtual_location = base_address.as_u64() + index * 8;
        unsafe { ptr::read_volatile(virtual_location as *const u64) }
    }

    fn hpet_write(base_address: VirtAddr, index: u64, value: u64) {
        let virtual_location = base_address.as_u64() + index * 8;
        unsafe { ptr::write_volatile(virtual_location as *mut u64, value) };
    }
}
