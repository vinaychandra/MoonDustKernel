use core::{panic, task::Poll};

use alloc::{boxed::Box, sync::Arc};
use moondust_utils::{id_generator::IdGenerator, sync::mutex::Mutex};
use x86_64::structures::paging::PageTable;

use crate::arch::globals;
use crate::arch::memory::kernel_page_table::KernelPageTable;
use crate::common::memory::paging::{IMemoryMapper, MapperPermissions};

use super::{
    state::{Registers, ThreadState},
    user_future::UserSwitcher,
};

#[derive(Debug)]
pub struct Thread {
    pub thread_id: usize,
    page_table: Arc<Mutex<KernelPageTable>>,
    pub state: ThreadState,
}

static THREAD_ID_GENERATOR: IdGenerator = IdGenerator::new();

impl Thread {
    pub async fn new_empty_process(stack_start: u64, stack_size: usize) -> Self {
        let mut thread = Self {
            thread_id: THREAD_ID_GENERATOR.get_value(),
            page_table: Arc::new(Mutex::new(KernelPageTable::new(
                Self::create_new_kernel_only_pagetable_from_current(),
            ))),
            state: ThreadState::NotStarted(Registers::default()),
        };
        thread.setup_user_stack(stack_start, stack_size).await;
        thread
            .increase_user_heap(globals::USER_HEAP_DEFAULT_SIZE)
            .await
            .unwrap();
        thread
    }

    pub async fn new_empty_thread(&self, stack_start: u64, stack_size: usize) -> Self {
        let mut thread = Self {
            thread_id: THREAD_ID_GENERATOR.get_value(),
            page_table: self.page_table.clone(),
            state: ThreadState::NotStarted(Registers::default()),
        };
        thread.setup_user_stack(stack_start, stack_size).await;
        thread
            .increase_user_heap(globals::USER_HEAP_DEFAULT_SIZE)
            .await
            .unwrap();
        thread
    }

    pub async fn run_thread(mut self) -> u8 {
        loop {
            self.activate().await;
            let user_switcher = UserSwitcher { thread: &mut self };
            user_switcher.await;

            match self.state {
                ThreadState::Running => panic!("Thread cannot be in Running state after running!"),
                ThreadState::NotStarted(_) => panic!("Thread cannot be NotStarted after running!"),
                ThreadState::Syscall(_) => {
                    if let Poll::Ready(ret_val) = self.process_syscall().await {
                        return ret_val;
                    }
                }
            }
        }
    }

    pub fn setup_user_ip(&mut self, ip: u64) {
        if let ThreadState::NotStarted(registers) = &mut self.state {
            registers.rip = ip;
        } else {
            panic!("Cannot setup ip when threadstate is not in NotStarted state.")
        }
    }

    async fn setup_user_stack(&mut self, stack_start: u64, stack_size: usize) {
        debug_assert!(
            stack_size % globals::PAGE_SIZE == 0,
            "Stack size should be aligned"
        );

        let mut kpt = self.page_table.lock().await;
        kpt.map_with_alloc(
            stack_start as *const u8,
            stack_size,
            MapperPermissions::READ | MapperPermissions::RING_3 | MapperPermissions::WRITE,
        )
        .unwrap();
        if let ThreadState::NotStarted(registers) = &mut self.state {
            registers.rbp = stack_start + stack_size as u64;
            registers.rsp = stack_start + stack_size as u64;
        } else {
            panic!("Cannot setup user stack when threadstate is not in syscall.")
        }
    }

    async fn increase_user_heap(
        &mut self,
        size_to_increase: usize,
    ) -> Result<(usize, usize), &'static str> {
        let mut kpt = self.page_table.lock().await;
        kpt.map_more_user_heap(size_to_increase)
    }

    pub fn get_page_table(&self) -> &Arc<Mutex<KernelPageTable>> {
        &self.page_table
    }

    pub async fn activate(&mut self) {
        let mut pt = self.page_table.lock().await;

        ::x86_64::instructions::interrupts::without_interrupts(|| {
            // This will also prevent the page table from being dropped.
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

impl Drop for Thread {
    fn drop(&mut self) {
        THREAD_ID_GENERATOR.return_value(self.thread_id);
    }
}
