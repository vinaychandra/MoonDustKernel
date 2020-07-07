pub const BOOTBOOT_MAGIC: &'static [u8; 5usize] = b"BOOT\0";
pub const PROTOCOL_MINIMAL: u32 = 0;
pub const PROTOCOL_STATIC: u32 = 1;
pub const PROTOCOL_DYNAMIC: u32 = 2;
pub const PROTOCOL_BIGENDIAN: u32 = 128;
pub const LOADER_BIOS: u32 = 0;
pub const LOADER_UEFI: u32 = 4;
pub const LOADER_RPI: u32 = 8;
pub const FB_ARGB: u32 = 0;
pub const FB_RGBA: u32 = 1;
pub const FB_ABGR: u32 = 2;
pub const FB_BGRA: u32 = 3;
pub const MMAP_USED: u32 = 0;
pub const MMAP_FREE: u32 = 1;
pub const MMAP_ACPI: u32 = 2;
pub const MMAP_MMIO: u32 = 3;
pub const INITRD_MAXSIZE: u32 = 16;
#[repr(C, packed)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct MMapEnt {
    pub ptr: u64,
    pub size: u64,
}
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct BOOTBOOT {
    pub magic: [u8; 4usize],
    pub size: u32,
    pub protocol: u8,
    pub fb_type: u8,
    pub numcores: u16,
    pub bspid: u16,
    pub timezone: i16,
    pub datetime: [u8; 8usize],
    pub initrd_ptr: u64,
    pub initrd_size: u64,
    pub fb_ptr: *mut u8,
    pub fb_size: u32,
    pub fb_width: u32,
    pub fb_height: u32,
    pub fb_scanline: u32,
    pub arch: BOOTBOOT__bindgen_ty_1,
    pub mmap: MMapEnt,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union BOOTBOOT__bindgen_ty_1 {
    pub x86_64: BOOTBOOT__bindgen_ty_1__bindgen_ty_1,
    pub aarch64: BOOTBOOT__bindgen_ty_1__bindgen_ty_2,
    _bindgen_union_align: [u64; 8usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BOOTBOOT__bindgen_ty_1__bindgen_ty_1 {
    pub acpi_ptr: u64,
    pub smbi_ptr: u64,
    pub efi_ptr: u64,
    pub mp_ptr: u64,
    pub unused0: u64,
    pub unused1: u64,
    pub unused2: u64,
    pub unused3: u64,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BOOTBOOT__bindgen_ty_1__bindgen_ty_2 {
    pub acpi_ptr: u64,
    pub mmio_ptr: u64,
    pub efi_ptr: u64,
    pub unused0: u64,
    pub unused1: u64,
    pub unused2: u64,
    pub unused3: u64,
    pub unused4: u64,
}
extern "C" {
    pub static mut bootboot: BOOTBOOT;
}
extern "C" {
    pub static mut environment: *mut custom_ctypes::c_uchar;
}
extern "C" {
    pub static mut fb: u8;
}
#[doc = " Display text on screen *"]
#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct psf2_t {
    pub magic: u32,
    pub version: u32,
    pub headersize: u32,
    pub flags: u32,
    pub numglyph: u32,
    pub bytesperglyph: u32,
    pub height: u32,
    pub width: u32,
    pub glyphs: u8,
}
extern "C" {
    pub static mut _binary_font_psf_start: custom_ctypes::c_uchar;
}
pub mod custom_ctypes {
    pub type c_int = i64;
    pub type c_uchar = u64;
}

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
