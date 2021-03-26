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

    // returns added [start, end)
    pub fn expand_heap_by(increase_by: usize) -> (u64, u64) {
        let expand_by = Syscalls::Heap(HeapControl::IncreaseHeapBy(increase_by));
        let rval = expand_by.invoke();
        if let Sysrets::SuccessWithVal2(a, b) = rval {
            return (a, b);
        } else {
            panic!("Heap expansion failed!");
        }
    }
}
