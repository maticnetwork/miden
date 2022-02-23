use super::{
    AuxiliaryTableTrace, Bitwise, Felt, FieldElement, Hasher, Memory, Process, StackTrace,
    AUXILIARY_TABLE_WIDTH, STACK_TOP_SIZE,
};
use core::slice;
use winterfell::Trace;

// VM EXECUTION TRACE
// ================================================================================================

/// TODO: for now this consists only of stack trace and auxiliary traces, but will need to
/// include decoder trace, etc.
pub struct ExecutionTrace {
    meta: Vec<u8>,
    stack: StackTrace,
    #[allow(dead_code)]
    aux_table: AuxiliaryTableTrace,
}

impl ExecutionTrace {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Builds an execution trace for the provided process.
    pub(super) fn new(process: Process) -> Self {
        let Process {
            system,
            decoder: _,
            stack,
            hasher,
            bitwise,
            memory,
            advice: _,
            proc_stack: _,
        } = process;

        // get the length required to hold all execution trace steps
        let aux_trace_len = hasher.trace_len() + bitwise.trace_len() + memory.trace_len();
        let mut trace_len = usize::max(stack.trace_len(), aux_trace_len);
        // pad the trace length to the next power of 2
        if !trace_len.is_power_of_two() {
            trace_len = trace_len.next_power_of_two();
        }

        // allocate columns for the trace of the auxiliary table
        // note: it may be possible to optimize this by initializing with Felt::zeroed_vector,
        // depending on how the compiler reduces Felt(0) and whether initializing here + iterating
        // to update selector values is faster than using resize to initialize all values
        let mut aux_table_trace: AuxiliaryTableTrace = (0..AUXILIARY_TABLE_WIDTH)
            .map(|_| Vec::<Felt>::with_capacity(trace_len))
            .collect::<Vec<_>>()
            .try_into()
            .expect("failed to convert vector to array");

        // fill the aux table with the column selectors and stacked coprocessor traces
        fill_aux_columns(&mut aux_table_trace, trace_len, hasher, bitwise, memory);

        // finalize stack trace and extend it to match the length of the auxiliary trace, if needed
        let step = system.clk();
        let mut stack_trace = stack.into_trace();
        for column in stack_trace.iter_mut() {
            finalize_column(column, step, trace_len);
        }

        Self {
            meta: Vec::new(),
            stack: stack_trace,
            aux_table: aux_table_trace,
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// TODO: add docs
    pub fn init_stack_state(&self) -> [Felt; STACK_TOP_SIZE] {
        let mut result = [Felt::ZERO; STACK_TOP_SIZE];
        self.read_row_into(0, &mut result);
        result
    }

    /// TODO: add docs
    pub fn last_stack_state(&self) -> [Felt; STACK_TOP_SIZE] {
        let mut result = [Felt::ZERO; STACK_TOP_SIZE];
        self.read_row_into(self.length() - 1, &mut result);
        result
    }

    // ACCESSORS FOR TESTING
    // --------------------------------------------------------------------------------------------
    #[cfg(test)]
    pub fn aux_table(&self) -> &AuxiliaryTableTrace {
        &self.aux_table
    }

    #[cfg(test)]
    pub fn stack(&self) -> &StackTrace {
        &self.stack
    }
}

// TRACE TRAIT IMPLEMENTATION
// ================================================================================================

impl Trace for ExecutionTrace {
    type BaseField = Felt;

    fn width(&self) -> usize {
        self.stack.len()
    }

    fn length(&self) -> usize {
        self.stack[0].len()
    }

    fn get(&self, col_idx: usize, row_idx: usize) -> Self::BaseField {
        self.stack[col_idx][row_idx]
    }

    fn meta(&self) -> &[u8] {
        &self.meta
    }

    fn read_row_into(&self, step: usize, target: &mut [Self::BaseField]) {
        for (i, register) in self.stack.iter().enumerate() {
            target[i] = register[step];
        }
    }

    fn into_columns(self) -> Vec<Vec<Self::BaseField>> {
        self.stack.into()
    }
}

// TRACE FRAGMENT
// ================================================================================================

/// TODO: add docs
pub struct TraceFragment<'a> {
    data: Vec<&'a mut [Felt]>,
}

impl<'a> TraceFragment<'a> {
    /// Creates a new TraceFragment with its data allocated to the specified capacity.
    pub fn new(capacity: usize) -> Self {
        TraceFragment {
            data: Vec::with_capacity(capacity),
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns the number of columns in this execution trace fragment.
    pub fn width(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of rows in this execution trace fragment.
    pub fn len(&self) -> usize {
        self.data[0].len()
    }

    // DATA MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Updates a single cell in this fragment with provided value.
    #[inline(always)]
    pub fn set(&mut self, row_idx: usize, col_idx: usize, value: Felt) {
        self.data[col_idx][row_idx] = value;
    }

    /// Returns a mutable iterator the the columns of this fragment.
    pub fn columns(&mut self) -> slice::IterMut<'_, &'a mut [Felt]> {
        self.data.iter_mut()
    }

    /// Adds a new column to this fragment by pushing a mutable slice with the first `len`
    /// elements of the provided column. Returns the rest of the provided column as a separate
    /// mutable slice.
    pub fn push_column_slice(&mut self, column: &'a mut [Felt], len: usize) -> &'a mut [Felt] {
        let (column_fragment, rest) = column.split_at_mut(len);
        self.data.push(column_fragment);
        rest
    }

    // TEST METHODS
    // --------------------------------------------------------------------------------------------

    #[cfg(test)]
    pub fn trace_to_fragment(trace: &'a mut [Vec<Felt>]) -> Self {
        let mut data = Vec::new();
        for column in trace.iter_mut() {
            data.push(column.as_mut_slice());
        }
        Self { data }
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Copies the final output value down to the end of the stack trace, then extends the column to
/// the length of the execution trace, if it's longer than the stack trace, and copies the last
/// value to the end of that as well.
fn finalize_column(column: &mut Vec<Felt>, step: usize, trace_len: usize) {
    let last_value = column[step];
    column[step..].fill(last_value);
    column.resize(trace_len, last_value);
}

/// Fills the provided auxiliary table trace with the stacked execution traces of the Hasher,
/// Bitwise, and Memory coprocessors, along with selector columns to identify each coprocessor
/// trace and padding to fill the rest of the table.
///
/// The auxiliary trace table can be thought of as 4 stacked segments in the following form:
/// * Hasher segment: contains the hasher trace and selector *
/// This segment fills the first rows of the table up to the length of the hasher `trace_len`.
/// - column 0: selector column with values set to ZERO
/// - columns 1-17: execution trace of hasher coprocessor
///
/// * Bitwise segment: contains the bitwise trace and selectors *
/// This segment begins at the end of the hasher segment and fills the next rows of the table for
/// the `trace_len` of the bitwise coprocessor.
/// - column 0: selector column with values set to ONE
/// - column 1: selector column with values set to ZERO
/// - columns 2-14: execution trace of bitwise coprocessor
/// - columns 15-17: unused columns padded with ZERO
///
/// * Memory segment: contains the memory trace and selectors *
/// This segment begins at the end of the bitwise segment and fills the next rows of the table for
/// the `trace_len` of the memory coprocessor.
/// - column 0-1: selector columns with values set to ONE
/// - column 2: selector column with values set to ZERO
/// - columns 3-16: execution trace of memory coprocessor
/// - column 17: unused column padded with ZERO
///
/// * Final segment: unused *
/// This segment begins at the end of the memory segment and fills the rest of the rows in the table
/// up to the full length of the execution trace.
/// - columns 0-2: selector columns with values set to ONE
/// - columns 3-17: unused columns padded with ZERO
///
fn fill_aux_columns(
    aux_table_trace: &mut AuxiliaryTableTrace,
    trace_len: usize,
    hasher: Hasher,
    bitwise: Bitwise,
    memory: Memory,
) {
    // allocate fragments to be filled with the respective execution traces of each coprocessor
    let mut hasher_fragment = TraceFragment::new(AUXILIARY_TABLE_WIDTH);
    let mut bitwise_fragment = TraceFragment::new(AUXILIARY_TABLE_WIDTH);
    let mut memory_fragment = TraceFragment::new(AUXILIARY_TABLE_WIDTH);

    // set the selectors and padding as required by each column and segment
    // and add the hasher, bitwise, and memory segments to their respective fragments
    // so they can be filled with the coprocessor traces
    for (column_num, column) in aux_table_trace.iter_mut().enumerate() {
        match column_num {
            0 => {
                // set the selector value for the hasher segment to ZERO
                column.resize(hasher.trace_len(), Felt::ZERO);
                // set the selector value for all other segments ONE
                column.resize(trace_len, Felt::ONE);
            }
            1 => {
                // initialize hasher segment and set bitwise segment selector value to ZERO
                column.resize(hasher.trace_len() + bitwise.trace_len(), Felt::ZERO);
                // set selector value for all other segments to ONE
                column.resize(trace_len, Felt::ONE);
                // add hasher segment to the hasher fragment to be filled from the hasher trace
                hasher_fragment.push_column_slice(column, hasher.trace_len());
            }
            2 => {
                // initialize hasher and bitwise segments and set memory segment selector to ZERO
                column.resize(
                    hasher.trace_len() + bitwise.trace_len() + memory.trace_len(),
                    Felt::ZERO,
                );
                // set selector value for the final segment to ONE
                column.resize(trace_len, Felt::ONE);
                // add hasher segment to the hasher fragment to be filled from the hasher trace
                let rest_of_column = hasher_fragment.push_column_slice(column, hasher.trace_len());
                // add bitwise segment to the bitwise fragment to be filled from the bitwise trace
                bitwise_fragment.push_column_slice(rest_of_column, bitwise.trace_len());
            }
            15 | 16 => {
                // initialize hasher & memory segments and pad bitwise & final segments with ZERO
                column.resize(trace_len, Felt::ZERO);
                // add hasher segment to the hasher fragment to be filled from the hasher trace
                let rest_of_column = hasher_fragment.push_column_slice(column, hasher.trace_len());
                // split the column again to skip the bitwise segment, which has already been padded
                let (_, rest_of_column) = rest_of_column.split_at_mut(bitwise.trace_len());
                // add memory segment to the memory fragment to be filled from the memory trace
                memory_fragment.push_column_slice(rest_of_column, memory.trace_len());
            }
            17 => {
                // initialize hasher segment and pad bitwise, memory, and final segments with ZERO
                column.resize(trace_len, Felt::ZERO);
                // add hasher segment to the hasher fragment to be filled from the hasher trace
                hasher_fragment.push_column_slice(column, hasher.trace_len());
            }
            _ => {
                // initialize hasher, bitwise, memory segments and pad the final segment with ZERO
                column.resize(trace_len, Felt::ZERO);
                // add hasher segment to the hasher fragment to be filled from the hasher trace
                let rest_of_column = hasher_fragment.push_column_slice(column, hasher.trace_len());
                // add bitwise segment to the bitwise fragment to be filled from the bitwise trace
                let rest_of_column =
                    bitwise_fragment.push_column_slice(rest_of_column, bitwise.trace_len());
                // add memory segment to the memory fragment to be filled from the memory trace
                memory_fragment.push_column_slice(rest_of_column, memory.trace_len());
            }
        }
    }

    // fill the fragments with the execution trace from each coprocessor
    // TODO: this can be parallelized to fill the traces in multiple threads
    hasher.fill_trace(&mut hasher_fragment);
    bitwise.fill_trace(&mut bitwise_fragment);
    memory.fill_trace(&mut memory_fragment);
}
