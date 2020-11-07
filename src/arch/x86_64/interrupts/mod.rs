use self::{
    keyboard::keyboard_handler,
    timer::{hpet_timer_handler, timer_interrupt_handler},
};

use super::{gdt, globals};
use acpi::{AcpiTables, InterruptModel};
use alloc::vec::Vec;
use apic::{io_apic::IoApicBase, ApicBase};
use spin::Mutex;
use x86_64::{
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
    VirtAddr,
};

pub mod keyboard;
pub mod timer;

/// Offsets for PIC raised interrupts. 32 is the first value available
/// after the inbuilt CPU exceptions. This is for the main PIC.
const PIC_OFFSET: u8 = 32;

/// Index of interrupts. This is the index where IRQs are raised
/// on PIC.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_OFFSET,
    Keyboard = 33,
    Spurious,
    Error,
    HpetTimer = 36, // 36
}

impl InterruptIndex {
    /// Get the index in IRQ list for the given interrupt.
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Convert the index to usize.
    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();
static OFFSET: Mutex<VirtAddr> = Mutex::new(VirtAddr::new_truncate(0));

/// Initialize interrupts
/// - Disable PIC
/// - Enable APIC/xAPIC
/// - Enable HPET
pub fn initialize(phys_mem_offset: VirtAddr) {
    unsafe {
        IDT.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        IDT.page_fault.set_handler_fn(page_fault_handler);

        IDT[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_handler);
        IDT[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        IDT[InterruptIndex::HpetTimer.as_usize()].set_handler_fn(hpet_timer_handler);

        IDT.load();
    }

    *OFFSET.lock() = phys_mem_offset;
}

#[thread_local]
pub static mut LAPIC: Option<ApicBase> = None;

pub unsafe fn load_interrupts() -> Result<(), &'static str> {
    info!(target:"interrupts", "Setting up interrupts");

    info!(target:"interrupts", "Disabling PIC");
    super::devices::pic8259_simple::simple_pic::disable_pic();
    info!(target:"interrupts", "PIC disabled");

    info!(target:"interrupts", "Load ACPI tables");
    let acpi_addr = crate::bootboot::bootboot.arch.x86_64.acpi_ptr as *const u8 as usize;
    info!(target:"interrupts", "ACPI Addr is {:x}", acpi_addr);

    let offset = OFFSET.lock().as_u64() as usize;
    let handler = crate::common::devices::acpi::MemoryHandler::new(offset);
    let acpi_tables =
        AcpiTables::from_rsdt(handler, 2, acpi_addr).or(Err("ACPI Tables cannot be parsed."))?;
    info!(target:"interrupts", "ACPI tables loaded successfully.");

    let platform_info = acpi_tables.platform_info().unwrap();
    if let InterruptModel::Apic(apic) = platform_info.interrupt_model {
        let mut apic_base =
            ApicBase::new((apic.local_apic_address + globals::MEM_MAP_LOCATION) as *mut ());
        info!(
            target: "interrupts",
            "APIC is at {:x} with ID {:?} and version {:?}",
            apic.local_apic_address,
            apic_base.id().read(),
            apic_base.version().read()
        );

        // Enabling APIC logic.
        let mut spu = apic_base.spurious_interrupt_vector();
        let mut val = spu.read();
        val.enable_apic_software(true);
        spu.write(val);

        info!(target: "interrupts", "APIC enabled: {}", apic_base.spurious_interrupt_vector().read().apic_software_enabled());

        let mut ioapics: Vec<IoApicBase> = apic
            .io_apics
            .iter()
            .map(|ioa| IoApicBase::new((ioa.address as usize + offset) as *mut u8))
            .collect();
        let first_ioapic = &mut ioapics[0];
        let lapic_id = apic_base.id().read().id();

        LAPIC = Some(apic_base);

        let mut redirection_entry = first_ioapic.read_redirection_table_entry(1);
        redirection_entry.set_destination(lapic_id);
        redirection_entry.set_vector(33);
        first_ioapic.write_redirection_table_entry(1, redirection_entry);

        let mut redirection_entry = first_ioapic.read_redirection_table_entry(8);
        redirection_entry.set_destination(lapic_id);
        redirection_entry.set_vector(36);
        first_ioapic.write_redirection_table_entry(8, redirection_entry);

        info!(
            target: "interrupts",
            "Number of IoApics found: {}", ioapics.len());
    } else {
        return Err("APIC data not found in ACPI tables.");
    }

    x86_64::instructions::interrupts::enable();

    Ok(())
}

/// Handler than be used for non-standard faults.
#[allow(dead_code)]
extern "x86-interrupt" fn unhandled_fault(stack_frame: &mut InterruptStackFrame, error_code: u64) {
    error!(
        target: "unhandled_fault",
        "EXCEPTION: Unhandled FAULT\n{:#?}\nError Code: {}",
        stack_frame,
        error_code
    );
    crate::arch::hlt_loop();
}

/// Handler than be used for non-standard faults.
#[allow(dead_code)]
extern "x86-interrupt" fn unhandled_fault_noerr(stack_frame: &mut InterruptStackFrame) {
    error!(target: "unhandled_fault_noerr", "EXCEPTION: Unhandled FAULT\n{:#?}", stack_frame);
    crate::arch::hlt_loop();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) -> ! {
    error!(
        target: "DoubleFaultHandler",
        "EXCEPTION: DOUBLE FAULT\n{:#?}\nError Code: {}",
        stack_frame,
        error_code
    );
    crate::arch::hlt_loop()
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    error!(
        target: "PageFaultHandler",
        "EXCEPTION: PAGE FAULT\n{:#?}\nError Code: {:?}\nAccessed Address: {:?}",
        stack_frame,
        error_code,
        Cr2::read()
    );
    crate::arch::hlt_loop()
}
