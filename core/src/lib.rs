pub mod hasher;
pub mod program;
pub mod utils;
pub use math::{fields::f64::BaseElement as Felt, FieldElement, StarkField};

mod operations;
pub use operations::{AdviceInjector, DebugOptions, Operation, ProcInfo};

mod inputs;
pub use inputs::{AdviceSet, ProgramInputs};

pub mod errors;

// TYPE ALIASES
// ================================================================================================

pub type Word = [Felt; 4];

// CONSTANTS
// ================================================================================================

/// Number of stack registers which can be accesses by the VM directly.
pub const STACK_TOP_SIZE: usize = 16;
