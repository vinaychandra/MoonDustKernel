//! Simplified USTAR / TAR File format support.

use alloc::str;
use core::{fmt::Display, slice};

/// Representation of the header of an entry in an archive
#[repr(C)]
#[allow(missing_docs)]
#[allow(dead_code)]
struct UstarHeader {
    /// File name without a slash.
    pub name: [u8; 100],
    /// File mode
    pub mode: [u8; 8],
    /// User ID
    pub uid: [u8; 8],
    /// Group ID
    pub gid: [u8; 8],
    /// Size in bytes (octal)
    pub size: [u8; 12],
    /// Latest modification time
    pub mtime: [u8; 12],
    /// File and header checksum
    pub cksum: [u8; 8],
    /// File type (Link indicator)
    pub typeflag: EntryType,
    /// Linked path name or file name
    pub linkname: [u8; 100],

    // UStar format
    /// Forat representation for tar
    pub magic: [u8; 6],
    /// Version representation for tar
    pub version: [u8; 2],
    /// User name
    pub uname: [u8; 32],
    /// Group name
    pub gname: [u8; 32],
    /// Major device representation
    pub dev_major: [u8; 8],
    /// Minor device representation
    pub dev_minor: [u8; 8],
    /// Path name without trailing slashes
    pub prefix: [u8; 155],

    /// Padding to make the header 512 bytes.
    pub pad: [u8; 12],
}

// See https://en.wikipedia.org/wiki/Tar_%28computing%29#UStar_format
/// Indicate for the type of file described by a header.
///
/// A non-exhaustive enum representing the possible entry types
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
#[allow(dead_code)]
pub enum EntryType {
    /// Regular file
    Regular = 48,
    Regular2 = 5,
    /// Hard link
    Link = 49,
    /// Symbolic link
    Symlink = 50,
    /// Character device
    Char = 51,
    /// Block device
    Block = 52,
    /// Directory
    Directory = 53,
    /// Named pipe (fifo)
    Fifo = 54,
    /// Hints that destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

pub struct UStarArchive {
    file: &'static [UstarHeader],
}

impl UStarArchive {
    pub fn new(file_pointer: *const u8, size: usize) -> Self {
        assert!(size % 512 == 0);
        UStarArchive {
            file: unsafe { slice::from_raw_parts(file_pointer as *const UstarHeader, size / 512) },
        }
    }

    pub fn lookup(&self, file_name: &str) -> Option<&[u8]> {
        for file in self.entry_enumerator() {
            if file_name.eq_ignore_ascii_case(file.0) {
                return file.2;
            }
        }

        if file_name.starts_with("./") {
            return self.lookup(&file_name[2..]);
        }

        None
    }

    pub fn entry_enumerator(
        &self,
    ) -> impl Iterator<Item = (&'static str, EntryType, Option<&'static [u8]>)> {
        EntryIterators::new(self.file)
    }
}

impl Display for UStarArchive {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut s = f.debug_struct("UStarArchive");
        for entry in self.entry_enumerator() {
            let data = entry.2;
            let value = if data == None { 0 } else { data.unwrap().len() };
            s.field(entry.0, &format!("{:?} - {} KiB", entry.1, value / 1024));
        }

        s.finish()
    }
}

struct EntryIterators {
    file: &'static [UstarHeader],
    index: usize,
}

impl EntryIterators {
    fn new(file: &'static [UstarHeader]) -> Self {
        Self { file, index: 0 }
    }
}

impl Iterator for EntryIterators {
    type Item = (&'static str, EntryType, Option<&'static [u8]>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.file.len() {
            return None;
        }

        while alloc::str::from_utf8(&self.file[self.index].magic)
            .unwrap()
            .trim_matches(char::from(0))
            .trim()
            .eq("ustar")
        {
            let entry = &self.file[self.index];
            let index = self.index;

            let file_size = oct2bin(&entry.size[0..11]);
            self.index += ((file_size + 511) / 512) + 1;

            return Some((
                alloc::str::from_utf8(&entry.name)
                    .expect("Unexpected file name!")
                    .trim_matches(char::from(0)),
                entry.typeflag,
                if file_size > 0 {
                    Some(unsafe {
                        slice::from_raw_parts(
                            &self.file[index + 1] as *const UstarHeader as *const u8,
                            file_size as usize,
                        )
                    })
                } else {
                    None
                },
            ));
        }

        None
    }
}

fn oct2bin(data: &'static [u8]) -> usize {
    let mut n = 0 as usize;
    for digit in data {
        n *= 8;
        n += *digit as usize - 48; // ASCII octal. 48 is '0'
    }
    n
}
