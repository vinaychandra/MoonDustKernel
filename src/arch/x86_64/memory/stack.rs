use crate::common::memory::stack::Stack;

impl Stack {
    /// Switch to this stack.
    /// WARN: DO NOT ACCESS LOCAL VARIABLES AFTER CALLING THIS.
    #[inline(always)]
    pub fn switch_to(&self) {
        unsafe {
            asm!("
                mov rsp, {0}
                mov rbp, {0}
            ", in(reg) self.get_high_addr());
        }
    }
}
