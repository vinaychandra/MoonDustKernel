use super::{HeapControl, Syscalls, Sysrets};

pub struct Heap;

impl Heap {
    pub fn get_current_heap_size() -> usize {
        let get_heap_size = Syscalls::Heap(HeapControl::GetCurrentHeapSize);
        let rval = get_heap_size.invoke();
        if let Sysrets::SuccessWithVal(value) = rval {
            return value as usize;
        } else {
            unreachable!("Sysret here will be a SuccessWithVal")
        }
    }
}
