use crate::common::process::task::Task;

impl Task {
    /// Activate the task and run.
    #[inline(always)]
    pub fn activate(&self) {
        // self.stack.switch_to();
    }
}
