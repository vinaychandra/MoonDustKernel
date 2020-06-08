pub mod executor;
mod id_generator;
mod task;

pub use task::Task;

use id_generator::IdGenerator;

lazy_static! {
    /// TaskIdGenerator for kernel tasks.
    pub static ref TASK_ID_GENERATOR: IdGenerator = IdGenerator::new(65535, 128);
}
