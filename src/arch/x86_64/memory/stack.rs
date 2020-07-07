use crate::common::memory::stack::Stack;

impl Stack {
    /// Switch to this stack.
    /// WARN: DO NOT ACCESS LOCAL VARIABLES AFTER CALLING THIS.
    #[inline(always)]
    pub fn switch_to(&self) {
        let ptrs = self.get_stack_pointers();
        unsafe {
            asm!("
                mov rsp, {0}
                mov rbp, {1}
            ", in(reg) ptrs.1, in(reg) ptrs.0);
        }
    }
}
