#[derive(Debug, Clone)]
#[repr(C)]
pub enum SyscallInfo {
    Exit { val: u8 },
    Test { val: u8 },
}

#[derive(Debug, Clone)]
#[repr(C)]
pub enum SysretInfo {
    NoVal,
}

impl SyscallInfo {
    /// Invoke the syscall.
    pub fn invoke(self) {
        unsafe {
            #[cfg(target_feature = "sse")]
            {
                asm!(
                    "syscall",
                    in("rdi") &self,
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
                    in("rdi") &self,
                    // All caller-saved registers must be marked as clobberred
                    out("rax") _, out("rcx") _, out("rdx") _, out("rsi") _,
                    out("r8") _, out("r9") _, out("r10") _, out("r11") _,
                )
            }
        }
    }
}
