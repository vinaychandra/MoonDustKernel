use super::{
    super::memory::{
        paging::{IMemoryMapper, IPageTable},
        stack::Stack,
    },
    elf_loader::DefaultElfLoader,
    task::Task,
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::ops::Deref;
use elfloader::ElfBinary;
use spin::RwLock;

pub struct Process {
    pub pcb: Arc<RwLock<ProcessControlBlock>>,
    pub tasks: RwLock<Vec<Arc<Task>>>,
}

impl Process {
    pub fn new_with_pcb(pcb: ProcessControlBlock) -> Process {
        Process {
            pcb: Arc::new(RwLock::new(pcb)),
            tasks: RwLock::new(Vec::new()),
        }
    }

    /// Load a ELF into memory at `vbase` and return the entry point.
    pub fn load_elf(&self, vbase: u64, binary: ElfBinary) -> Result<u64, &'static str> {
        let mut pcb = self.pcb.write();
        let mut loader = DefaultElfLoader::new(vbase, pcb.mapper.as_mut());
        binary.load(&mut loader)?;
        Ok(binary.entry_point())
    }
}

pub struct ProcessControlBlock {
    pub mapper: Box<dyn IMemoryMapper>,
    pub kernel_stack: Stack,
    pub page_table: Box<dyn IPageTable>,
}

impl Deref for Process {
    type Target = RwLock<ProcessControlBlock>;
    fn deref(&self) -> &Self::Target {
        &self.pcb
    }
}
