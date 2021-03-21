use alloc::{boxed::Box, sync::Arc};
use moondust_utils::sync::mutex::Mutex;
use x86_64::structures::paging::PageTable;

use crate::arch::globals;
use crate::arch::memory::paging::KernelPageTable;
use crate::common::memory::paging::{IMemoryMapper, MapperPermissions};

use super::state::ThreadState;

#[derive(Debug)]
pub struct Thread {
    pub thread_id: usize,
    page_table: Arc<Mutex<KernelPageTable>>,
    pub state: ThreadState,

    stack_start: u64,
    stack_size: usize,
    pub(super) is_stack_setup: bool,
}

impl Thread {
    pub async fn new_empty_process(thread_id: usize, stack_start: u64, stack_size: usize) -> Self {
        let mut r = Self {
            thread_id,
            page_table: Arc::new(Mutex::new(KernelPageTable::new(
                Self::create_new_kernel_only_pagetable_from_current(),
            ))),
            state: ThreadState::Syscall {
                registers: Default::default(),
                syscall_info: None,
                sysret_data: None,
            },
            stack_start,
            stack_size,
            is_stack_setup: false,
        };
        r.setup_user_stack().await;
        r
    }

    pub fn setup_user_ip(&mut self, ip: u64) {
        if let ThreadState::Syscall {
            registers,
            syscall_info: _,
            sysret_data: _,
        } = &mut self.state
        {
            registers.rip = ip;
        } else {
            panic!("Cannot setup ip when threadstate is not in syscall.")
        }
    }

    pub(super) async fn setup_user_stack(&mut self) {
        debug_assert!(
            self.stack_size % globals::PAGE_SIZE == 0,
            "Stack size should be aligned"
        );

        let mut kpt = self.page_table.lock().await;
        let mut mapper = kpt.get_mapper();
        mapper
            .map_with_alloc(
                self.stack_start as *const u8,
                self.stack_size,
                MapperPermissions::READ | MapperPermissions::RING_3 | MapperPermissions::WRITE,
            )
            .unwrap();
        if let ThreadState::Syscall {
            registers,
            syscall_info: _,
            sysret_data: _,
        } = &mut self.state
        {
            registers.rbp = self.stack_start + self.stack_size as u64;
            registers.rsp = self.stack_start + self.stack_size as u64;
            self.is_stack_setup = true;
        } else {
            panic!("Cannot setup user stack when threadstate is not in syscall.")
        }
    }
    pub fn get_page_table(&self) -> &Arc<Mutex<KernelPageTable>> {
        &self.page_table
    }

    pub async fn activate(&mut self) {
        let mut pt = self.page_table.lock().await;

        ::x86_64::instructions::interrupts::without_interrupts(|| {
            pt.activate();
            crate::arch::cpu_locals::CURRENT_PAGE_TABLE.replace(Some(self.page_table.clone()));
        });
    }

    fn create_new_kernel_only_pagetable_from_current() -> Box<PageTable> {
        let mut new_table = Box::new(PageTable::new());

        let table = unsafe { crate::arch::memory::active_level_4_table_default() };

        // Copy kernel level entries
        new_table[510] = table[510].clone(); // Direct mapping data
        new_table[511] = table[511].clone(); // Everything else.

        new_table
    }
}
