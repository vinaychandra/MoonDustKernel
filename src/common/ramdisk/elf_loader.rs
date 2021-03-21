use elfloader::{ElfLoader, Flags, LoadableHeaders, Rela, TypeRela64, VAddr, P64};

use crate::arch::globals;
pub use crate::common;

use self::common::memory::paging::{IMemoryMapper, MapperPermissions};
pub struct DefaultElfLoader<'a> {
    vbase: u64,
    mapper: &'a mut dyn IMemoryMapper,

    last_exe_section_location: u64,
}

impl<'a> DefaultElfLoader<'a> {
    pub fn new(vbase: u64, mapper: &'a mut dyn IMemoryMapper) -> DefaultElfLoader<'a> {
        DefaultElfLoader {
            vbase,
            mapper,
            last_exe_section_location: 0,
        }
    }

    pub fn get_exe_location(&self) -> u64 {
        self.last_exe_section_location
    }
}

/// Implement this trait for customized ELF loading.
///
/// The flow of ElfBinary is that it first calls `allocate` for all regions
/// that need to be allocated (i.e., the LOAD program headers of the ELF binary),
/// then `load` will be called to fill the allocated regions, and finally
/// `relocate` is called for every entry in the RELA table.
impl<'a> ElfLoader for DefaultElfLoader<'a> {
    /// Allocates a virtual region of `size` bytes at address `base`.
    fn allocate(&mut self, load_headers: LoadableHeaders) -> Result<(), &'static str> {
        for header in load_headers {
            info!(
                target:"elf",
                "allocate base = {:#x} size = {:#x} flags = {}",
                header.virtual_addr(),
                header.mem_size(),
                header.flags()
            );

            let virt_addr_to_load = header.virtual_addr();
            let target_vaddr = common::align_down(virt_addr_to_load as usize, globals::PAGE_SIZE);

            // We load only Ring 3 ELFs. So, add Ring3 permissions as well.
            let mut target_permissions = MapperPermissions::READ | MapperPermissions::RING_3;
            let perms = header.flags();
            if perms.is_write() {
                target_permissions |= MapperPermissions::WRITE;
            }
            if perms.is_execute() {
                target_permissions |= MapperPermissions::EXECUTE;
            }

            let size = common::align_up(header.mem_size() as usize, globals::PAGE_SIZE) as usize;
            self.mapper
                .map_with_alloc(target_vaddr as *const u8, size, target_permissions)?;

            // Zero the data
            for i in 0..size {
                let target_paddr = self
                    .mapper
                    .virt_to_phys((target_vaddr + i) as *const ())
                    .unwrap();
                let vaddr_in_current = target_paddr as u64 + globals::MEM_MAP_OFFSET_LOCATION;
                unsafe { *(vaddr_in_current as *mut u8) = 0 };
            }
        }

        Ok(())
    }

    /// Copies `region` into memory starting at `base`.
    /// The caller makes sure that there was an `allocate` call previously
    /// to initialize the region.
    fn load(&mut self, flags: Flags, base: VAddr, region: &[u8]) -> Result<(), &'static str> {
        let start = self.vbase + base;
        let end = self.vbase + base + region.len() as u64;
        info!(target:"elf", "load region into = {:#x} -- {:#x}", start, end);

        if flags.is_execute() {
            self.last_exe_section_location = start;
        }

        for i in 0..end - start {
            // Because we load everything in a target mapper rather than the current one, we use the mapper provided
            // for getting target locations.
            // TODO: Reduce virt_to_phys calls.
            let target_physical_addr = self.mapper.virt_to_phys((start + i) as *const ()).unwrap();
            let virt_addr_in_current =
                target_physical_addr as u64 + globals::MEM_MAP_OFFSET_LOCATION;
            unsafe { *(virt_addr_in_current as *mut u8) = region[i as usize] };
        }

        Ok(())
    }

    /// Request for the client to relocate the given `entry`
    /// within the loaded ELF file.
    fn relocate(&mut self, entry: &Rela<P64>) -> Result<(), &'static str> {
        let elf_entry_type = TypeRela64::from(entry.get_type());
        let target_vaddr = (self.vbase + entry.get_offset()) as *const ();
        let target_paddr = self.mapper.virt_to_phys(target_vaddr).unwrap();
        let vaddr_in_current = target_paddr as u64 + globals::MEM_MAP_OFFSET_LOCATION;

        // https://www.intezer.com/blog/elf/executable-and-linkable-format-101-part-3-relocations/
        match elf_entry_type {
            TypeRela64::R_RELATIVE => {
                // This is a relative relocation, add the offset (where we put our
                // binary in the vspace) to the addend and we're done.
                debug!(target:"elf",
                    "R_RELATIVE *{:p} = {:#x}",
                    target_vaddr,
                    self.vbase + entry.get_addend()
                );

                unsafe { *(vaddr_in_current as *mut u64) = self.vbase + entry.get_addend() };

                Ok(())
            }
            _ => Err("Unexpected relocation encountered"),
        }
    }
}
