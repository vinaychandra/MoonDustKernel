use core::fmt::{Debug, Error, Formatter};
use core::mem::MaybeUninit;
use core::ptr::{read_volatile, write_volatile};

#[repr(packed)]
pub struct Mmio<T> {
    value: MaybeUninit<T>,
}

impl<T> Mmio<T> {
    /// Create a new Mmio without initializing
    pub fn new() -> Self {
        Mmio {
            value: MaybeUninit::uninit(),
        }
    }

    pub fn read(&self) -> T {
        unsafe { read_volatile(self.value.as_ptr()) }
    }

    pub fn write(&mut self, value: T) {
        unsafe { write_volatile(self.value.as_mut_ptr(), value) };
    }
}

impl<T> Debug for Mmio<T>
where
    T: Copy + Debug,
{
    /// Debug volatilely reads `value`.
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        fmt.debug_struct("Mmio")
            .field("value", &self.read())
            .finish()
    }
}
