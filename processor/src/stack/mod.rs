use super::{
    BTreeMap, Felt, FieldElement, ProgramInputs, StackTopState, Vec, MIN_STACK_DEPTH,
    NUM_STACK_HELPER_COLS, STACK_TRACE_WIDTH, ZERO,
};
use core::cmp;

mod trace;
use trace::StackTrace;

mod overflow;
use overflow::OverflowTable;
pub use overflow::{OverflowTableRow, OverflowTableUpdate};

mod aux_trace;
pub use aux_trace::AuxTraceBuilder;

#[cfg(test)]
mod tests;

// CONSTANTS
// ================================================================================================

/// The last stack index accessible by the VM.
const MAX_TOP_IDX: usize = MIN_STACK_DEPTH - 1;

// STACK
// ================================================================================================

/// Stack for the VM.
///
/// This component is responsible for managing the state of the VM's stack, as well as building an
/// execution trace for all stack transitions.
///
/// The state is separated into two parts: the top 16 slots of the stack (stored in the `trace`
/// member), and items which don't fit into the top 16 slots (stored in the `overflow` member).
///
/// The depth of the stack can never drop below 16. If an item is removed from the stack when the
/// depth is 16, a ZERO element is inserted into the 16th slot.
///
/// ## Execution trace
/// The stack execution trace consists of 19 columns as illustrated below:
///
///   s0   s1   s2        s13   s14   s15   b0   b1   h0
/// ├────┴────┴────┴ ... ┴────┴─────┴─────┴────┴────┴────┤
///
/// The meaning of the above columns is as follows:
/// - s0...s15 are the columns representing the top 16 slots of the stack.
/// - Bookkeeping column b0 contains the number of items on the stack (i.e., the stack depth).
/// - Bookkeeping column b1 contains an address of a row in the “overflow table” in which we’ll
///   store the data that doesn’t fit into the top 16 slots. When b1=0, it means that all stack
///   data fits into the top 16 slots of the stack.
/// - Helper column h0 is used to ensure that stack depth does not drop below 16. Values in this
///   column are set by the prover non-deterministically to 1 / (b0−16) when b0 != 16, and to any
///   other value otherwise.
pub struct Stack {
    clk: usize,
    trace: StackTrace,
    overflow: OverflowTable,
    depth: usize,
}

impl Stack {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a [Stack] initialized with the specified program inputs.
    pub fn new(
        inputs: &ProgramInputs,
        init_trace_capacity: usize,
        keep_overflow_trace: bool,
    ) -> Self {
        Self {
            clk: 0,
            trace: StackTrace::new(inputs, init_trace_capacity),
            overflow: OverflowTable::new(keep_overflow_trace),
            depth: MIN_STACK_DEPTH,
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns depth of the stack at the current clock cycle.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the current clock cycle of the execution trace.
    pub fn current_clk(&self) -> usize {
        self.clk
    }

    /// Returns execution trace length for this stack.
    ///
    /// Trace length of the stack is equal to the number of cycles executed by the VM.
    pub fn trace_len(&self) -> usize {
        self.clk
    }

    /// Returns a copy of the item currently at the top of the stack.
    pub fn peek(&self) -> Felt {
        self.trace.peek_at(self.clk)
    }

    /// Return n items from the top of the stack, including the values from the overflow table.
    ///
    /// If n is None, this returns the entire stack.
    pub fn get_values(&self, n: Option<usize>) -> Vec<Felt> {
        let n = n.unwrap_or(usize::MAX);
        let num_items = cmp::min(n, self.depth());

        let num_top_items = cmp::min(MIN_STACK_DEPTH, num_items);
        let mut result = self.trace.get_stack_values_at(self.clk, num_top_items);

        if num_items > MIN_STACK_DEPTH {
            let num_overflow_items = num_items - MIN_STACK_DEPTH;
            self.overflow.append_into(&mut result, num_overflow_items);
        }

        result
    }

    /// Returns an execution trace of the top 16 stack slots and helper columns as a single array
    /// together with hints to be used in construction of stack-related auxiliary trace segment
    /// columns.
    ///
    /// If the stack trace is smaller than the specified `trace_len`, last value in each column is
    /// duplicated until the length of the columns reaches `trace_len`.
    ///
    /// `num_rand_rows` indicates the number of rows at the end of the trace which will be
    /// overwritten with random values. This parameter is unused because last rows are just
    /// duplicates of the prior rows and thus can be safely overwritten.
    pub fn into_trace(self, trace_len: usize, num_rand_rows: usize) -> super::StackTrace {
        let clk = self.current_clk();
        // make sure that only the duplicate rows will be overwritten with random values
        assert!(
            clk + num_rand_rows <= trace_len,
            "target trace length too small"
        );

        // fill in all trace columns after the last clock cycle with the value at the last clock
        // cycle
        let mut trace = self.trace.into_array();
        for column in trace.iter_mut() {
            let last_value = column[clk];
            column[clk..].fill(last_value);
            column.resize(trace_len, last_value);
        }

        super::StackTrace {
            trace,
            aux_builder: self.overflow.into_aux_builder(),
        }
    }

    /// Returns stack state at the specified clock cycle.
    ///
    /// This includes the stack + overflow entries.
    pub fn get_state_at(&self, clk: usize) -> Vec<Felt> {
        let mut result = self.trace.get_stack_state_at(clk).to_vec();
        self.overflow.append_state_into(&mut result, clk);

        result
    }

    // TRACE ACCESSORS AND MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Returns the value located at the specified position on the stack at the current clock cycle.
    pub fn get(&self, pos: usize) -> Felt {
        debug_assert!(pos < MIN_STACK_DEPTH, "stack underflow");
        self.trace.get_stack_value_at(self.clk, pos)
    }

    /// Sets the value at the specified position on the stack at the next clock cycle.
    pub fn set(&mut self, pos: usize, value: Felt) {
        debug_assert!(pos < MIN_STACK_DEPTH, "stack underflow");
        self.trace.set_stack_value_at(self.clk + 1, pos, value);
    }

    /// Copies stack values starting at the specified position at the current clock cycle to the
    /// same position at the next clock cycle.
    pub fn copy_state(&mut self, start_pos: usize) {
        debug_assert!(
            start_pos < MIN_STACK_DEPTH,
            "start cannot exceed stack top size"
        );
        self.trace.copy_stack_state_at(self.clk, start_pos);
    }

    /// Copies stack values starting at the specified position at the current clock cycle to
    /// position - 1 at the next clock cycle.
    ///
    /// If the stack depth is greater than 16, an item is moved from the overflow stack to the
    /// "in-memory" portion of the stack. If the stack depth is 16, the 16th element of the
    /// stack is set to ZERO.
    pub fn shift_left(&mut self, start_pos: usize) {
        debug_assert!(start_pos > 0, "start position must be greater than 0");
        debug_assert!(
            start_pos < MIN_STACK_DEPTH,
            "start position cannot exceed stack top size"
        );

        match self.depth {
            0..=MAX_TOP_IDX => unreachable!("stack underflow"),
            MIN_STACK_DEPTH => {
                // Shift in a ZERO, to prevent depth shrinking below the minimum stack depth.
                self.trace.stack_shift_left_at(self.clk, start_pos, ZERO);
                self.trace.copy_helpers_at(self.clk);
            }
            _ => {
                // Update the stack & overflow table.
                let (from_overflow, prev_addr) = self.overflow.pop(self.clk);
                self.trace
                    .stack_shift_left_at(self.clk, start_pos, from_overflow);

                // Update the bookkeeping & helper columns.
                self.trace.helpers_shift_left_at(self.clk, prev_addr);

                // Stack depth only decreases when it is greater than the minimum stack depth.
                self.depth -= 1;
            }
        }
    }

    /// Copies stack values starting at the specified position at the current clock cycle to
    /// position + 1 at the next clock cycle
    ///
    /// If stack depth grows beyond 16 items, the additional item is pushed into the overflow table.
    pub fn shift_right(&mut self, start_pos: usize) {
        debug_assert!(
            start_pos < MIN_STACK_DEPTH,
            "start position cannot exceed stack top size"
        );

        // Update the stack.
        self.trace.stack_shift_right_at(self.clk, start_pos);

        // Update the overflow table.
        let to_overflow = self.trace.get_stack_value_at(self.clk, MAX_TOP_IDX);
        self.overflow.push(to_overflow, self.clk);

        // Update the bookkeeping & helper columns.
        self.trace.helpers_shift_right_at(self.clk);

        // Stack depth always increases on right shift.
        self.depth += 1;
    }

    /// Increments the clock cycle.
    pub fn advance_clock(&mut self) {
        self.clk += 1;
    }

    // UTILITY METHODS
    // --------------------------------------------------------------------------------------------

    /// Makes sure there is enough memory allocated for the trace to accommodate a new row.
    ///
    /// Trace length is doubled every time it needs to be increased.
    pub fn ensure_trace_capacity(&mut self) {
        self.trace.ensure_trace_capacity(self.clk);
    }

    // TEST HELPERS
    // --------------------------------------------------------------------------------------------

    /// Returns trace state at the current clock cycle.
    ///
    /// Trace state is always 16 elements long and contains the top 16 values of the stack. When
    /// the stack depth is less than 16, the un-used slots contain ZEROs.
    #[cfg(test)]
    pub fn trace_state(&self) -> StackTopState {
        self.trace.get_stack_state_at(self.clk)
    }
}
