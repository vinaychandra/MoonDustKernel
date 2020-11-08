//! Additional support files for BOOTBOOT providing usability wrappers.

use crate::bootboot::*;

// Custom Logic for BOOT BOOT
/*
#define MMapEnt_Ptr(a)  (a->ptr)
#define MMapEnt_Size(a) (a->size & 0xFFFFFFFFFFFFFFF0)
#define MMapEnt_Type(a) (a->size & 0xF)
#define MMapEnt_IsFree(a) ((a->size&0xF)==1)

#define MMAP_USED     0   /* don't use. Reserved or unknown regions */
#define MMAP_FREE     1   /* usable memory */
#define MMAP_ACPI     2   /* acpi memory, volatile and non-volatile as well */
#define MMAP_MMIO     3   /* memory mapped IO region */
*/
impl MMapEnt {
    pub fn ptr(&self) -> usize {
        self.ptr as usize
    }

    /// Size of memory area in bytes.
    pub fn size(&self) -> usize {
        (self.size & 0xFFFFFFFFFFFFFFF0) as usize
    }

    /// Returns true if the area can be used by OS.
    pub fn is_free(&self) -> bool {
        let is_free = (self.size & 0xF) == 1;
        is_free
    }

    /// Get the type of memory entry.
    pub fn get_type(&self) -> MMapEntType {
        let _ptr = self.ptr as *mut u8;
        let _size = self.size as *mut u8;
        match self.size & 0xF {
            0 => MMapEntType::Used,
            1 => MMapEntType::Free,
            2 => MMapEntType::Acpi,
            3 => MMapEntType::Mmio,
            _ => MMapEntType::Used,
        }
    }

    pub fn set_type(&mut self, entry: MMapEntType) {
        match entry {
            MMapEntType::Used => self.size |= 0x0,
            MMapEntType::Free => self.size |= 0x1,
            MMapEntType::Acpi => self.size |= 0x2,
            MMapEntType::Mmio => self.size |= 0x3,
        }
    }
}

impl BOOTBOOT {
    pub fn get_mmap_entries(&self) -> &'static [MMapEnt] {
        let num_mmap_entries = (self.size - 128) / 16;
        unsafe {
            core::slice::from_raw_parts(
                &self.mmap.ptr as *const u64 as *const MMapEnt,
                num_mmap_entries as usize,
            )
        }
    }
}

impl MMapEnt {
    pub fn end_address(&self) -> u64 {
        self.ptr + self.size() as u64
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum MMapEntType {
    Used,
    Free,
    Acpi,
    Mmio,
}
