use core::ptr;
use x86_64::registers::model_specific::Msr;
use x86_64::{PhysAddr, VirtAddr};

/// Location for APIC Base MSR.
const IA32_APIC_BASE: u32 = 0x1B;

/// The structure for Local APIC.
pub struct LApic {
    /// The base virtual address where LApic can
    /// be accessed.
    base_virt_address: u64,
}

impl LApic {
    /// Create the new LApic structure.
    /// `base_virtual_address` is the value at which LApic's
    /// memory mapped structures are located.
    /// `read_base` function can be sued to retrieve the physical
    /// address of this address.
    pub fn new(base_virtual_address: VirtAddr) -> LApic {
        LApic {
            base_virt_address: base_virtual_address.as_u64(),
        }
    }

    /// Get the LApic Base address.
    /// This function reads from `IA32_APIC_BASE`.
    pub unsafe fn read_base() -> PhysAddr {
        PhysAddr::new(Msr::new(IA32_APIC_BASE).read() & 0xFFFFFF000 as u64)
    }

    fn mem_read(virtual_location: u64) -> u32 {
        unsafe { ptr::read_volatile(virtual_location as *const u32) }
    }

    fn mem_write(virtual_location: u64, value: u32) {
        unsafe { ptr::write_volatile(virtual_location as *mut u32, value) };
    }

    /// Read the LApic register and return it's value.
    pub fn read(&self, register: LapicRegister) -> u32 {
        let target_location: u64 = self.base_virt_address + register.offset;
        Self::mem_read(target_location)
    }

    /// Write to the LApic register.
    pub fn write(&self, register: LapicRegister, value: u32) {
        let target_location: u64 = self.base_virt_address + register.offset;
        Self::mem_write(target_location, value);
    }

    /// Enable the LApic.
    pub fn enable(&self) {
        let mut val = self.read(SPURIOUS_VECTOR_REGISTER);
        val |= 1 << 8;
        self.write(SPURIOUS_VECTOR_REGISTER, val)
    }

    /// Given the interrupt vector index, this function sends EOI to
    /// the Local APIC.
    /// We check the nth bit of the ISR to check if its set; if so,
    /// we need to send the local APIC an EOI.
    pub fn send_eoi(&self, interrupt_index: u8) {
        let index_location: u64 = (interrupt_index / 32).into();
        let apic_location = index_location + 0x10;

        let value = self.read(LapicRegister::new(apic_location));

        let nth_bit = interrupt_index % 32;
        let check_nth_bit = 1 << nth_bit;
        let should_send_eoi = value & check_nth_bit > 0;
        if should_send_eoi {
            self.write(EOI_REGISTER, 1);
        }
    }

    /// Initialize a periodic APIC timer.
    /// ## Arguments
    /// - `target_vector`: The target vector in Interrupt Vector Table.
    /// ## Notes
    /// https://wiki.osdev.org/APIC_timer
    /// http://ethv.net/workshops/osdev/notes/notes-4
    pub fn initialize_apic_timer(&self, target_vector: u8) {
        let mut lvt = self.read(TIMER_LVT_REGISTER);
        // Clear target vector.
        lvt &= 0xFF_FF_FF_00;
        // Load target vector.
        lvt |= target_vector as u32;
        // Set bit 17 for periodic mode.
        lvt |= 1 << 16;
        // Disable initially (bit 16)
        lvt |= 1 << 15;
        self.write(TIMER_LVT_REGISTER, lvt);

        let mut dcr = self.read(DIVIDE_CONFIGURATION_REGISTER);
        // Clear divide
        dcr &= 0xFF_FF_FF_00 | 0b100;
        // Divide by 16 mode (set value to 0xb1010)
        dcr |= 0xb1010;
        self.write(DIVIDE_CONFIGURATION_REGISTER, dcr);

        // Set initial count
        self.write(INITIAL_COUNT_REGISTER, 30_000);

        // Unmask timer interrupt. Remove bit 16.
        lvt &= 0b1111_1111_1111_1111_0111_1111_1111_1111;
        self.write(TIMER_LVT_REGISTER, lvt);
    }
}

/// Structure for a register.
/// This is a simple way to provide strong typed access to target registers.
pub struct LapicRegister {
    offset: u64,
}

impl LapicRegister {
    const fn new(index: u64) -> LapicRegister {
        LapicRegister { offset: index << 4 }
    }
}

pub const APIC_ID_REGISTER: LapicRegister = LapicRegister::new(0x2);
pub const EOI_REGISTER: LapicRegister = LapicRegister::new(0xb);
pub const SPURIOUS_VECTOR_REGISTER: LapicRegister = LapicRegister::new(0xf);

pub const TIMER_LVT_REGISTER: LapicRegister = LapicRegister::new(0x32);
pub const DIVIDE_CONFIGURATION_REGISTER: LapicRegister = LapicRegister::new(0x3e);
pub const INITIAL_COUNT_REGISTER: LapicRegister = LapicRegister::new(0x38);

#[allow(dead_code)]
pub const CURRENT_COUNT_REGISTER: LapicRegister = LapicRegister::new(0x39);
