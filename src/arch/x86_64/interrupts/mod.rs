mod handlers;
mod hpet;
mod pic;

use super::gdt;
use crate::*;
use lazy_static::lazy_static;
use x86_64::{
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
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

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.page_fault.set_handler_fn(handlers::page_fault_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(handlers::double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(handlers::timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(handlers::keyboard_handler);
        idt[InterruptIndex::HpetTimer.as_usize()].set_handler_fn(handlers::hpet_timer_handler);

        idt
    };
}

/// Initialize interrupts
/// - Disable PIC
/// - Enable APIC/xAPIC
/// - Enable HPET
pub fn initialize(phys_mem_offset: VirtAddr) {
    IDT.load();

    // Disable legacy PIC
    pic::pic8259_simple::simple_pic::disable_pic();

    // Load ACPI tables
    let acpi = pic::acpi::load_acpi(phys_mem_offset);

    let interrupt_model = acpi.interrupt_model.unwrap();
    match interrupt_model {
        acpi::InterruptModel::Apic(apic) => {
            pic::xapic::initialize_apic(
                phys_mem_offset,
                apic.io_apics.first().expect("No IOAPICs found!").address,
            );
        }
        _ => panic!("Interrupt model not supported."),
    }

    // HPET
    let hpet_location = acpi.hpet.expect("Cannot find HPET");
    let location = phys_mem_offset + hpet_location.base_address;
    unsafe { hpet::init(location) };
}

/// Handler than be used for non-standard faults.
#[allow(dead_code)]
extern "x86-interrupt" fn unhandled_fault(stack_frame: &mut InterruptStackFrame, error_code: u64) {
    kernel_error!(
        "EXCEPTION: Unhandled FAULT\n{:#?}\nError Code: {}",
        stack_frame,
        error_code
    );
    crate::arch::hlt_loop();
}

/// Handler than be used for non-standard faults.
#[allow(dead_code)]
extern "x86-interrupt" fn unhandled_fault_noerr(stack_frame: &mut InterruptStackFrame) {
    kernel_error!("EXCEPTION: Unhandled FAULT\n{:#?}", stack_frame);
    crate::arch::hlt_loop();
}
