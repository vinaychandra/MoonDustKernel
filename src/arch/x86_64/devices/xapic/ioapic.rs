use core::ptr;
use x86_64::VirtAddr;

/// Structure representing IOApic.
pub struct IOApic {
    index_register_location: u64,
    data_register_location: u64,
}

impl IOApic {
    /// Create a new IOApic.
    /// The base register is where I/O Apic is mapped
    /// to. This location should be discovered depending
    /// on the environment.
    ///
    /// For example, both qemu and Bochs always map the
    /// I/O APIC to address 0xfec00000.
    pub fn new(base_register_location: VirtAddr) -> IOApic {
        IOApic {
            index_register_location: base_register_location.as_u64(),
            data_register_location: base_register_location.as_u64() + 0x10,
        }
    }

    fn mem_read(virtual_location: u64) -> u32 {
        unsafe { ptr::read_volatile(virtual_location as *const u32) }
    }

    fn mem_write(virtual_location: u64, value: u32) {
        unsafe { ptr::write_volatile(virtual_location as *mut u32, value) };
    }

    /// Read from the IOApic register.
    pub fn read(&self, index: u32) -> u32 {
        Self::mem_write(self.index_register_location, index);
        Self::mem_read(self.data_register_location)
    }

    /// Write to the IOApic register.
    pub fn write(&self, index: u32, value: u32) {
        Self::mem_write(self.index_register_location, index);
        Self::mem_write(self.data_register_location, value);
    }

    /// Remap the IRQ to a target interrupt vector.
    /// `irq` : The IRQ to route.
    /// `vector` : The target vector to route to.
    /// `apic_id`: The APIC id that this IRQ is mapped in.
    pub fn set_irq(&self, irq: u32, apic_id: u32, vector: u32) {
        let low_index: u32 = 0x10 + irq * 2;
        let high_index: u32 = 0x10 + irq * 2 + 1;

        let mut high: u32 = self.read(high_index);
        // set APIC ID
        high = high & !0xff000000;
        high = high | (apic_id << 24);
        self.write(high_index, high);

        let mut low: u32 = self.read(low_index);

        // unmask the IRQ
        low &= !(1 << 16);

        // set to physical delivery mode
        low &= !(1 << 11);

        // set to fixed delivery mode
        low &= !0x700;

        // set delivery vector
        low &= !0xff;
        low |= vector;

        self.write(low_index, low);
    }
}
