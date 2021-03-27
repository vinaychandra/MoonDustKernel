pub mod heap;

#[derive(Debug, Clone)]
#[repr(C)]
pub enum Syscalls<'a> {
    Exit(u8),
    Debug { data: &'a str },

    Heap(HeapControl),
    Process(ProcessControl),
}

#[derive(Debug, Clone)]
#[repr(C)]
pub enum HeapControl {
    GetCurrentHeapSize,
    IncreaseHeapBy(usize),
}

#[derive(Debug, Clone)]
#[repr(C)]
pub enum ProcessControl {
    CreateThread {
        ip: usize,
        stack_size: usize,
        extra_data: u64,
    },
}

#[derive(Debug, Clone)]
#[repr(C)]
pub enum Sysrets {
    NoVal,
    Fail,
    SuccessWithVal(u64),
    SuccessWithVal2(u64, u64),
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct SyscallWrapper<'a> {
    pub call_info: Syscalls<'a>,
    pub return_info: Sysrets,
}

impl Syscalls<'_> {
    /// Invoke the syscall.
    pub fn invoke(self) -> Sysrets {
        let mut wrapper = SyscallWrapper {
            call_info: self,
            return_info: Sysrets::NoVal,
        };

        unsafe {
            #[cfg(target_feature = "sse")]
            {
                asm!(
                    "syscall",
                    in("rdi") &mut wrapper,
                    // All caller-saved registers must be marked as clobberred
                    out("rax") _, out("rcx") _, out("rdx") _, out("rsi") _,
                    out("r8") _, out("r9") _, out("r10") _, out("r11") _,
                    out("xmm0") _, out("xmm1") _, out("xmm2") _, out("xmm3") _,
                    out("xmm4") _, out("xmm5") _, out("xmm6") _, out("xmm7") _,
                    out("xmm8") _, out("xmm9") _, out("xmm10") _, out("xmm11") _,
                    out("xmm12") _, out("xmm13") _, out("xmm14") _, out("xmm15") _,
                )
            }

            #[cfg(not(target_feature = "sse"))]
            {
                asm!(
                    "syscall",
                    in("rdi") &mut wrapper,
                    // All caller-saved registers must be marked as clobberred
                    out("rax") _, out("rcx") _, out("rdx") _, out("rsi") _,
                    out("r8") _, out("r9") _, out("r10") _, out("r11") _,
                )
            }
        }

        wrapper.return_info
    }
}
