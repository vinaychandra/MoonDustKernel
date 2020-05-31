// Alternative APIC code
pub fn initialize_apic(phys_mem_offset: u64) {
    let phy_mem_read = |physical_location: u64| -> u32 {
        unsafe { core::ptr::read_volatile((physical_location + phys_mem_offset) as *const u32) }
    };

    let phy_mem_write = |physical_location: u64, value: u32| {
        unsafe {
            core::ptr::write_volatile((physical_location + phys_mem_offset) as *mut u32, value)
        };
    };

    let apic_read = |offset: u64| -> u32 { phy_mem_read(0xfee00000u64 + offset) };
    let apic_write = |offset: u64, value: u32| phy_mem_write(0xfee00000u64 + offset, value);

    let ioapic_read = |offset: u32| -> u32 {
        phy_mem_write(0xfec00000, offset);
        phy_mem_read(0xfec00000 + 0x10)
    };

    let ioapic_write = |offset: u32, value: u32| {
        phy_mem_write(0xfec00000, offset);
        phy_mem_write(0xfec00000 + 0x10, value);
    };

    // Enable local APIC
    let val = apic_read(0xf0);
    let val = val | (1 << 8);
    apic_write(0xf0, val);

    // get apic id
    let apic_id = apic_read(0x20);

    let ioapic_set_irq = |irq: u32, apic_id: u32, vector: u32| {
        let low_index: u32 = 0x10 + irq * 2;
        let high_index: u32 = 0x10 + irq * 2 + 1;

        let mut high: u32 = ioapic_read(high_index);
        // set APIC ID
        high = high & !0xff000000;
        high = high | (apic_id << 24);
        ioapic_write(high_index, high);

        let mut low: u32 = ioapic_read(low_index);

        // unmask the IRQ
        low &= !(1 << 16);

        // set to physical delivery mode
        low &= !(1 << 11);

        // set to fixed delivery mode
        low &= !0x700;

        // set delivery vector
        low &= !0xff;
        low |= vector;

        ioapic_write(low_index, low);
    };

    ioapic_set_irq(2, apic_id, 32);
    ioapic_set_irq(1, apic_id, 33);

    unsafe { init_pit() };
}

unsafe fn init_pit() {
    let mut a = x86_64::instructions::port::Port::<u32>::new(0x43);
    let mut b = x86_64::instructions::port::Port::<u32>::new(0x40);
    a.write(0x34);
    b.write(0xa9);
    b.write(0x04);
}
