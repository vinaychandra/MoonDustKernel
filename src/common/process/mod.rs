mod elf_loader;
pub mod id_generator;
mod process;
pub mod single_core_executor;
pub mod task;

use id_generator::IdGenerator;
pub use process::*;

lazy_static! {
    /// TaskIdGenerator for kernel tasks
    pub static ref TASK_ID_GENERATOR: IdGenerator = IdGenerator::new(65535, 128);
}
