use alloc::{boxed::Box, sync::Arc};
use moondust_utils::sync::mutex::Mutex;
use x86_64::structures::paging::PageTable;

use crate::arch::memory::paging::KernelPageTable;

#[derive(Debug)]
pub struct Thread {
    page_table: Arc<Mutex<KernelPageTable>>,
}

impl Thread {
    pub fn new_empty_process() -> Self {
        Self {
            page_table: Arc::new(Mutex::new(KernelPageTable::new(
                Self::create_new_kernel_only_pagetable_from_current(),
            ))),
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
