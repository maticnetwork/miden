#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

use core::ops::Range;

pub mod chiplets;
pub use chiplets::hasher;
pub mod decoder;
pub mod errors;
pub mod range;

pub use math::{fields::f64::BaseElement as Felt, ExtensionOf, FieldElement, StarkField};

mod program;
pub use program::{blocks as code_blocks, Library, Program};

mod operations;
pub use operations::{
    AdviceInjector, AssemblyOp, Decorator, DecoratorIterator, DecoratorList, Operation, ProcMarker,
};

mod inputs;
pub use inputs::{AdviceSet, ProgramInputs};

pub mod utils;
use utils::range;

// TYPE ALIASES
// ================================================================================================

pub type Word = [Felt; 4];

pub type StackTopState = [Felt; MIN_STACK_DEPTH];

// CONSTANTS
// ================================================================================================

/// Field element representing ZERO in the base field of the VM.
pub const ZERO: Felt = Felt::ZERO;

/// Field element representing ONE in the base field of the VM.
pub const ONE: Felt = Felt::ONE;

/// The minimum length of the execution trace. This is the minimum required to support range checks.
pub const MIN_TRACE_LEN: usize = 1024;

/// The minimum stack depth enforced by the VM. This is also the number of stack registers which can
/// be accessed by the VM directly.
pub const MIN_STACK_DEPTH: usize = 16;

/// Number of bookkeeping and helper columns in the stack trace.
pub const NUM_STACK_HELPER_COLS: usize = 3;

// MAIN TRACE LAYOUT
// ------------------------------------------------------------------------------------------------

//      system          decoder           stack      range checks    chiplets
//    (2 columns)     (22 columns)    (19 columns)    (4 columns)     (18 columns)
// ├───────────────┴───────────────┴───────────────┴───────────────┴─────────────────┤

pub const SYS_TRACE_OFFSET: usize = 0;
pub const SYS_TRACE_WIDTH: usize = 2;
pub const SYS_TRACE_RANGE: Range<usize> = range(SYS_TRACE_OFFSET, SYS_TRACE_WIDTH);

pub const CLK_COL_IDX: usize = SYS_TRACE_OFFSET;
pub const FMP_COL_IDX: usize = SYS_TRACE_OFFSET + 1;

// decoder trace
pub const DECODER_TRACE_OFFSET: usize = SYS_TRACE_OFFSET + SYS_TRACE_WIDTH;
pub const DECODER_TRACE_WIDTH: usize = 22;
pub const DECODER_TRACE_RANGE: Range<usize> = range(DECODER_TRACE_OFFSET, DECODER_TRACE_WIDTH);

// Stack trace
pub const STACK_TRACE_OFFSET: usize = DECODER_TRACE_OFFSET + DECODER_TRACE_WIDTH;
pub const STACK_TRACE_WIDTH: usize = MIN_STACK_DEPTH + NUM_STACK_HELPER_COLS;
pub const STACK_TRACE_RANGE: Range<usize> = range(STACK_TRACE_OFFSET, STACK_TRACE_WIDTH);

// Range check trace
pub const RANGE_CHECK_TRACE_OFFSET: usize = STACK_TRACE_OFFSET + STACK_TRACE_WIDTH;
pub const RANGE_CHECK_TRACE_WIDTH: usize = 4;
pub const RANGE_CHECK_TRACE_RANGE: Range<usize> =
    range(RANGE_CHECK_TRACE_OFFSET, RANGE_CHECK_TRACE_WIDTH);

// Chiplets trace
pub const CHIPLETS_OFFSET: usize = RANGE_CHECK_TRACE_OFFSET + RANGE_CHECK_TRACE_WIDTH;
pub const CHIPLETS_WIDTH: usize = 18;
pub const CHIPLETS_RANGE: Range<usize> = range(CHIPLETS_OFFSET, CHIPLETS_WIDTH);

// Chiplets components
pub const MEMORY_TRACE_WIDTH: usize = 14;

pub const TRACE_WIDTH: usize = CHIPLETS_OFFSET + CHIPLETS_WIDTH;

// AUXILIARY COLUMNS LAYOUT
// ------------------------------------------------------------------------------------------------

//      decoder         stack       range checks      hasher      chiplets
//    (3 columns)     (1 column)     (3 columns)    (1 column)       (1 column)
// ├───────────────┴──────────────┴──────────────┴───────────────┴───────────────┤

// Decoder auxiliary columns
pub const DECODER_AUX_TRACE_OFFSET: usize = 0;
pub const DECODER_AUX_TRACE_WIDTH: usize = 3;
pub const DECODER_AUX_TRACE_RANGE: Range<usize> =
    range(DECODER_AUX_TRACE_OFFSET, DECODER_AUX_TRACE_WIDTH);

// Stack auxiliary columns
pub const STACK_AUX_TRACE_OFFSET: usize = DECODER_AUX_TRACE_RANGE.end;
pub const STACK_AUX_TRACE_WIDTH: usize = 1;
pub const STACK_AUX_TRACE_RANGE: Range<usize> =
    range(STACK_AUX_TRACE_OFFSET, STACK_AUX_TRACE_WIDTH);

// Range check auxiliary columns
pub const RANGE_CHECK_AUX_TRACE_OFFSET: usize = STACK_AUX_TRACE_RANGE.end;
pub const RANGE_CHECK_AUX_TRACE_WIDTH: usize = 3;
pub const RANGE_CHECK_AUX_TRACE_RANGE: Range<usize> =
    range(RANGE_CHECK_AUX_TRACE_OFFSET, RANGE_CHECK_AUX_TRACE_WIDTH);

// Chiplets auxiliary columns
pub const CHIPLETS_AUX_TRACE_OFFSET: usize = HASHER_AUX_TRACE_RANGE.end;
pub const CHIPLETS_AUX_TRACE_WIDTH: usize = 1;
pub const CHIPLETS_AUX_TRACE_RANGE: Range<usize> =
    range(CHIPLETS_AUX_TRACE_OFFSET, CHIPLETS_AUX_TRACE_WIDTH);

// Hasher auxiliary columns
pub const HASHER_AUX_TRACE_OFFSET: usize = RANGE_CHECK_AUX_TRACE_RANGE.end;
pub const HASHER_AUX_TRACE_WIDTH: usize = 1;
pub const HASHER_AUX_TRACE_RANGE: Range<usize> =
    range(HASHER_AUX_TRACE_OFFSET, HASHER_AUX_TRACE_WIDTH);

pub const AUX_TRACE_WIDTH: usize = CHIPLETS_AUX_TRACE_RANGE.end;

/// Number of random elements available to the prover after the commitment to the main trace
/// segment.
pub const AUX_TRACE_RAND_ELEMENTS: usize = 16;
