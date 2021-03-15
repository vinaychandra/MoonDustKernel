use x86_64::{
    instructions::{
        segmentation::{load_ss, set_cs},
        tables::load_tss,
    },
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use crate::common::align_down;

use super::globals;

const SPECIAL_STACK_SIZES: usize = 4096 * 5;

pub const DOUBLE_FAULT_IST_INDEX: usize = 0;
#[thread_local]
static DOUBLE_FAULT_STACK: [u8; SPECIAL_STACK_SIZES] = [0; SPECIAL_STACK_SIZES]; // TODO: Stack Protection.
#[thread_local]
static PRIVILEGE_0_STACK: [u8; SPECIAL_STACK_SIZES] = [0; SPECIAL_STACK_SIZES]; // TODO: Stack Protection.

#[thread_local]
static mut TSS: TaskStateSegment = TaskStateSegment::new();

#[thread_local]
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

#[thread_local]
static mut SELECTORS: SegmentSelectors = SegmentSelectors::new();

struct SegmentSelectors {
    kernel_code_selector: SegmentSelector,
    kernel_data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

impl SegmentSelectors {
    pub const fn new() -> SegmentSelectors {
        SegmentSelectors {
            kernel_code_selector: SegmentSelector(0),
            kernel_data_selector: SegmentSelector(0),
            user_code_selector: SegmentSelector(0),
            user_data_selector: SegmentSelector(0),
            tss_selector: SegmentSelector(0),
        }
    }
}

pub fn initialize_gdt() {
    unsafe {
        TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] =
            VirtAddr::new(get_stack_align_for_array(&DOUBLE_FAULT_STACK));
        TSS.privilege_stack_table[0] = VirtAddr::new(get_stack_align_for_array(&PRIVILEGE_0_STACK));

        let mut gdt = GlobalDescriptorTable::new();
        let kernel_code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let kernel_data_selector = gdt.add_entry(Descriptor::kernel_data_segment());

        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());

        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        GDT = gdt;

        SELECTORS = SegmentSelectors {
            kernel_code_selector,
            kernel_data_selector,
            user_code_selector,
            user_data_selector,
            tss_selector,
        };
        GDT.load();
        set_cs(SELECTORS.kernel_code_selector);
        load_ss(SELECTORS.kernel_data_selector);
        load_tss(SELECTORS.tss_selector);

        x86_64::registers::model_specific::Star::write(
            SELECTORS.user_code_selector,
            SELECTORS.user_data_selector,
            SELECTORS.kernel_code_selector,
            SELECTORS.kernel_data_selector,
        )
        .unwrap();
    }
}

fn get_stack_align_for_array<'a>(array: &'a [u8]) -> u64 {
    let last_entry_addr = &array[array.len() - 1] as *const u8 as usize;
    let high_aligned_addr = align_down(last_entry_addr, globals::STACK_ALIGN);
    high_aligned_addr as u64
}
