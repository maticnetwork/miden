use super::{
    chiplets::{AuxTraceBuilder as ChipletsAuxTraceBuilder, HasherAuxTraceBuilder},
    decoder::AuxTraceHints as DecoderAuxTraceHints,
    range::AuxTraceBuilder as RangeCheckerAuxTraceBuilder,
    stack::AuxTraceBuilder as StackAuxTraceBuilder,
    Digest, Felt, FieldElement, Process, StackTopState, Vec,
};
use vm_core::{
    decoder::{NUM_USER_OP_HELPERS, USER_OP_HELPERS_OFFSET},
    AUX_TRACE_RAND_ELEMENTS, AUX_TRACE_WIDTH, DECODER_TRACE_OFFSET, MIN_STACK_DEPTH, MIN_TRACE_LEN,
    STACK_TRACE_OFFSET, TRACE_WIDTH, ZERO,
};
use winterfell::{EvaluationFrame, Matrix, Serializable, Trace, TraceLayout};

#[cfg(feature = "std")]
use vm_core::StarkField;

mod utils;
pub use utils::{build_lookup_table_row_values, AuxColumnBuilder, LookupTableRow, TraceFragment};

mod decoder;

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================

/// Number of rows at the end of an execution trace which are injected with random values.
pub const NUM_RAND_ROWS: usize = 1;

// TYPE ALIASES
// ================================================================================================

type RandomCoin = vm_core::utils::RandomCoin<Felt, vm_core::hasher::Hasher>;

// VM EXECUTION TRACE
// ================================================================================================

pub struct AuxTraceHints {
    pub(crate) decoder: DecoderAuxTraceHints,
    pub(crate) stack: StackAuxTraceBuilder,
    pub(crate) range: RangeCheckerAuxTraceBuilder,
    pub(crate) hasher: HasherAuxTraceBuilder,
    pub(crate) chiplets: ChipletsAuxTraceBuilder,
}

/// Execution trace which is generated when a program is executed on the VM.
///
/// The trace consists of the following components:
/// - Main traces of System, Decoder, Operand Stack, Range Checker, and Auxiliary Co-Processor
///   components.
/// - Hints used during auxiliary trace segment construction.
/// - Metadata needed by the STARK prover.
pub struct ExecutionTrace {
    meta: Vec<u8>,
    layout: TraceLayout,
    main_trace: Matrix<Felt>,
    aux_trace_hints: AuxTraceHints,
    program_hash: Digest,
}

impl ExecutionTrace {
    // CONSTANTS
    // --------------------------------------------------------------------------------------------

    /// Number of rows at the end of an execution trace which are injected with random values.
    pub const NUM_RAND_ROWS: usize = NUM_RAND_ROWS;

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Builds an execution trace for the provided process.
    pub(super) fn new(process: Process) -> Self {
        // use program hash to initialize random element generator; this generator will be used
        // to inject random values at the end of the trace; using program hash here is OK because
        // we are using random values only to stabilize constraint degrees, and not to achieve
        // perfect zero knowledge.
        let program_hash: Digest = process.decoder.program_hash().into();
        let rng = RandomCoin::new(&program_hash.to_bytes());
        let (main_trace, aux_trace_hints) = finalize_trace(process, rng);

        Self {
            meta: Vec::new(),
            layout: TraceLayout::new(TRACE_WIDTH, [AUX_TRACE_WIDTH], [AUX_TRACE_RAND_ELEMENTS]),
            main_trace: Matrix::new(main_trace),
            aux_trace_hints,
            program_hash,
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns hash of the program execution of which resulted in this execution trace.
    pub fn program_hash(&self) -> Digest {
        self.program_hash
    }

    /// Returns the initial state of the top 16 stack registers.
    pub fn init_stack_state(&self) -> StackTopState {
        let mut result = [ZERO; MIN_STACK_DEPTH];
        for (i, result) in result.iter_mut().enumerate() {
            *result = self.main_trace.get_column(i + STACK_TRACE_OFFSET)[0];
        }
        result
    }

    /// Returns the final state of the top 16 stack registers.
    pub fn last_stack_state(&self) -> StackTopState {
        let last_step = self.last_step();
        let mut result = [ZERO; MIN_STACK_DEPTH];
        for (i, result) in result.iter_mut().enumerate() {
            *result = self.main_trace.get_column(i + STACK_TRACE_OFFSET)[last_step];
        }
        result
    }

    /// Returns helper registers state at the specified `clk` of the VM
    pub fn get_user_op_helpers_at(&self, clk: usize) -> [Felt; NUM_USER_OP_HELPERS] {
        let mut result = [ZERO; NUM_USER_OP_HELPERS];
        for (i, result) in result.iter_mut().enumerate() {
            *result = self
                .main_trace
                .get_column(DECODER_TRACE_OFFSET + USER_OP_HELPERS_OFFSET + i)[clk];
        }
        result
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    /// Returns the index of the last row in the trace.
    fn last_step(&self) -> usize {
        self.length() - NUM_RAND_ROWS - 1
    }

    // TEST HELPERS
    // --------------------------------------------------------------------------------------------
    #[cfg(feature = "std")]
    #[allow(dead_code)]
    pub fn print(&self) {
        let mut row = [ZERO; TRACE_WIDTH];
        for i in 0..self.length() {
            self.main_trace.read_row_into(i, &mut row);
            println!("{:?}", row.iter().map(|v| v.as_int()).collect::<Vec<_>>());
        }
    }

    #[cfg(test)]
    pub fn test_finalize_trace(process: Process) -> (Vec<Vec<Felt>>, AuxTraceHints) {
        let rng = RandomCoin::new(&[0; 32]);
        finalize_trace(process, rng)
    }
}

// TRACE TRAIT IMPLEMENTATION
// ================================================================================================

impl Trace for ExecutionTrace {
    type BaseField = Felt;

    fn layout(&self) -> &TraceLayout {
        &self.layout
    }

    fn length(&self) -> usize {
        self.main_trace.num_rows()
    }

    fn meta(&self) -> &[u8] {
        &self.meta
    }

    fn main_segment(&self) -> &Matrix<Felt> {
        &self.main_trace
    }

    fn build_aux_segment<E: FieldElement<BaseField = Felt>>(
        &mut self,
        aux_segments: &[Matrix<E>],
        rand_elements: &[E],
    ) -> Option<Matrix<E>> {
        // we only have one auxiliary segment
        if !aux_segments.is_empty() {
            return None;
        }

        // TODO: build auxiliary columns in multiple threads

        // add decoder's running product columns
        let decoder_aux_columns = decoder::build_aux_columns(
            &self.main_trace,
            &self.aux_trace_hints.decoder,
            rand_elements,
        );

        // add stack's running product columns
        let stack_aux_columns = self
            .aux_trace_hints
            .stack
            .build_aux_columns(&self.main_trace, rand_elements);

        // add the range checker's running product columns
        let range_aux_columns = self
            .aux_trace_hints
            .range
            .build_aux_columns(&self.main_trace, rand_elements);

        // add hasher's running product columns
        let hasher_aux_columns = self
            .aux_trace_hints
            .hasher
            .build_aux_columns(&self.main_trace, rand_elements);

        // add running product columns for the chiplets module
        let chiplets_aux_columns = self
            .aux_trace_hints
            .chiplets
            .build_aux_columns(&self.main_trace, rand_elements);

        // combine all auxiliary columns into a single vector
        let mut aux_columns = decoder_aux_columns
            .into_iter()
            .chain(stack_aux_columns)
            .chain(range_aux_columns)
            .chain(hasher_aux_columns)
            .chain(chiplets_aux_columns)
            .collect::<Vec<_>>();

        // inject random values into the last rows of the trace
        let mut rng = RandomCoin::new(&self.program_hash.to_bytes());
        for i in self.length() - NUM_RAND_ROWS..self.length() {
            for column in aux_columns.iter_mut() {
                column[i] = rng.draw().expect("failed to draw a random value");
            }
        }

        Some(Matrix::new(aux_columns))
    }

    fn read_main_frame(&self, row_idx: usize, frame: &mut EvaluationFrame<Felt>) {
        let next_row_idx = (row_idx + 1) % self.length();
        self.main_trace.read_row_into(row_idx, frame.current_mut());
        self.main_trace
            .read_row_into(next_row_idx, frame.next_mut());
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Converts a process into a set of execution trace columns for each component of the trace.
///
/// The process includes:
/// - Determining the length of the trace required to accommodate the longest trace column.
/// - Padding the columns to make sure all columns are of the same length.
/// - Inserting random values in the last row of all columns. This helps ensure that there
///   are no repeating patterns in each column and each column contains a least two distinct
///   values. This, in turn, ensures that polynomial degrees of all columns are stable.
fn finalize_trace(process: Process, mut rng: RandomCoin) -> (Vec<Vec<Felt>>, AuxTraceHints) {
    let (system, decoder, stack, mut range, chiplets) = process.to_components();

    let clk = system.clk();

    // trace lengths of system and stack components must be equal to the number of executed cycles
    assert_eq!(clk, system.trace_len(), "inconsistent system trace lengths");
    assert_eq!(
        clk,
        decoder.trace_len(),
        "inconsistent decoder trace length"
    );
    assert_eq!(clk, stack.trace_len(), "inconsistent stack trace lengths");

    // Add the range checks required by the chiplets to the range checker.
    chiplets.append_range_checks(&mut range);

    // Get the trace length required to hold all execution trace steps.
    let max_len = [clk, range.trace_len(), chiplets.trace_len()]
        .into_iter()
        .max()
        .expect("failed to get max of component trace lengths");

    // pad the trace length to the next power of two and ensure that there is space for the
    // rows to hold random values
    let trace_len = (max_len + NUM_RAND_ROWS).next_power_of_two();
    assert!(
        trace_len >= MIN_TRACE_LEN,
        "trace length must be at least {}, but was {}",
        MIN_TRACE_LEN,
        trace_len
    );

    // combine all trace segments into the main trace
    let system_trace = system.into_trace(trace_len, NUM_RAND_ROWS);
    let decoder_trace = decoder.into_trace(trace_len, NUM_RAND_ROWS);
    let stack_trace = stack.into_trace(trace_len, NUM_RAND_ROWS);
    let range_check_trace = range.into_trace(trace_len, NUM_RAND_ROWS);
    let chiplets_trace = chiplets.into_trace(trace_len, NUM_RAND_ROWS);

    let mut trace = system_trace
        .into_iter()
        .chain(decoder_trace.trace)
        .chain(stack_trace.trace)
        .chain(range_check_trace.trace)
        .chain(chiplets_trace.trace)
        .collect::<Vec<_>>();

    // inject random values into the last rows of the trace
    for i in trace_len - NUM_RAND_ROWS..trace_len {
        for column in trace.iter_mut() {
            column[i] = rng.draw().expect("failed to draw a random value");
        }
    }

    let aux_trace_hints = AuxTraceHints {
        decoder: decoder_trace.aux_trace_hints,
        stack: stack_trace.aux_builder,
        range: range_check_trace.aux_builder,
        hasher: chiplets_trace.hasher_aux_builder,
        chiplets: chiplets_trace.aux_builder,
    };

    (trace, aux_trace_hints)
}
