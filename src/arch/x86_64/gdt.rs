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

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// Per-CPU TSS
#[thread_local]
static mut TSS: TaskStateSegment = TaskStateSegment::new();

/// Per-CPU GDT
#[thread_local]
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

/// Per-CPU Segment selectors
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

/// Initialize GDT.
pub fn init() {
    // IDT init
    unsafe {
        TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096;
            let stack = vec![0u8; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(stack.as_ptr());
            core::mem::forget(stack);
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        TSS.privilege_stack_table[0] = {
            const STACK_SIZE: usize = 4096;
            let stack = vec![0u8; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(stack.as_ptr());
            core::mem::forget(stack);
            info!(target: "TSS", "ESP0 stack is from {:x}", (stack_start + STACK_SIZE).as_u64());
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

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
        setup_usermode_gdt();
    }
}

pub fn setup_usermode_gdt() {
    unsafe {
        x86_64::registers::model_specific::Star::write(
            SELECTORS.user_code_selector,
            SELECTORS.user_data_selector,
            SELECTORS.kernel_code_selector,
            SELECTORS.kernel_data_selector,
        )
        .unwrap();
    }
}

pub unsafe fn set_tss_esp0(stack: VirtAddr) {
    TSS.privilege_stack_table[0] = stack;
}
