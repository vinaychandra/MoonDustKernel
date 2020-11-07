use super::gdt;
use acpi::AcpiTables;
use spin::Mutex;
use x86_64::{
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
    VirtAddr,
};

/// Offsets for PIC raised interrupts. 32 is the first value available
/// after the inbuilt CPU exceptions. This is for the main PIC.
const PIC_OFFSET: u8 = 32;

/// Index of interrupts. This is the index where IRQs are raised
/// on PIC.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_OFFSET,
    Keyboard,
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
        IDT.load();
    }

    *OFFSET.lock() = phys_mem_offset;
}

pub unsafe fn load_interrupts() {
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
        AcpiTables::from_rsdt(handler, 2, acpi_addr).expect("ACPI Tables cannot be parsed.");
    info!(target:"interrupts", "ACPI tables loaded successfully.");
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
