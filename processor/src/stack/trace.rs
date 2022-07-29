use super::{
    Felt, FieldElement, ProgramInputs, StackTopState, Vec, MAX_TOP_IDX, MIN_STACK_DEPTH,
    NUM_STACK_HELPER_COLS, STACK_TRACE_WIDTH,
};
use crate::utils::get_trace_len;
use vm_core::StarkField;

// STACK TRACE
// ================================================================================================

/// Execution trace of the stack component.
///
/// The trace consists of 19 columns grouped logically as follows:
/// - 16 stack columns holding the top of the stack.
/// - 3 columns for bookkeeping and helper values that manage left and right shifts.
pub struct StackTrace {
    stack: [Vec<Felt>; MIN_STACK_DEPTH],
    helpers: [Vec<Felt>; NUM_STACK_HELPER_COLS],
}

impl StackTrace {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a [StackTrace] instantiated with empty vectors for all columns.
    pub fn new(inputs: &ProgramInputs, init_trace_capacity: usize) -> Self {
        // Initialize the stack.
        let init_values = inputs.stack_init();
        let mut stack: Vec<Vec<Felt>> = Vec::with_capacity(MIN_STACK_DEPTH);
        for i in 0..MIN_STACK_DEPTH {
            let mut column = Felt::zeroed_vector(init_trace_capacity);
            if i < init_values.len() {
                column[0] = init_values[i];
            }
            stack.push(column)
        }

        // Initialize the bookkeeping & helper columns.
        let mut b0 = Felt::zeroed_vector(init_trace_capacity);
        b0[0] = Felt::new(MIN_STACK_DEPTH as u64);
        let helpers: [Vec<Felt>; 3] = [
            b0,
            Felt::zeroed_vector(init_trace_capacity),
            Felt::zeroed_vector(init_trace_capacity),
        ];

        StackTrace {
            stack: stack
                .try_into()
                .expect("Failed to convert vector to an array"),
            helpers,
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    pub fn into_array(self) -> [Vec<Felt>; STACK_TRACE_WIDTH] {
        let mut trace = Vec::with_capacity(STACK_TRACE_WIDTH);
        trace.extend_from_slice(&self.stack);
        trace.extend_from_slice(&self.helpers);

        trace
            .try_into()
            .expect("Failed to convert vector to an array")
    }

    // STACK ACCESSORS AND MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Returns a copy of the item at the top of the stack at the specified clock cycle.
    #[inline(always)]
    pub fn peek_at(&self, clk: usize) -> Felt {
        self.stack[0][clk]
    }

    /// Returns the value located at the specified position on the stack at the specified clock
    /// cycle.
    #[inline(always)]
    pub fn get_stack_value_at(&self, clk: usize, pos: usize) -> Felt {
        self.stack[pos][clk]
    }

    /// Sets the value at the specified position on the stack at the specified cycle.
    #[inline(always)]
    pub fn set_stack_value_at(&mut self, clk: usize, pos: usize, value: Felt) {
        self.stack[pos][clk] = value;
    }

    /// Return the specified number of states from the top of the stack at the specified clock
    /// cycle.
    pub fn get_stack_values_at(&self, clk: usize, num_items: usize) -> Vec<Felt> {
        self.get_stack_state_at(clk)[..num_items].to_vec()
    }

    /// Returns the stack trace state at the specified clock cycle.
    ///
    /// Trace state is always 16 elements long and contains the top 16 values of the stack.
    pub fn get_stack_state_at(&self, clk: usize) -> StackTopState {
        let mut result = [Felt::ZERO; MIN_STACK_DEPTH];
        for (result, column) in result.iter_mut().zip(self.stack.iter()) {
            *result = column[clk];
        }
        result
    }

    /// Copies the stack values starting at the specified position at the specified clock cycle to
    /// the same position at the next clock cycle.
    pub fn copy_stack_state_at(&mut self, clk: usize, start_pos: usize) {
        debug_assert!(
            start_pos < MIN_STACK_DEPTH,
            "start cannot exceed stack top size"
        );
        for i in start_pos..MIN_STACK_DEPTH {
            self.stack[i][clk + 1] = self.stack[i][clk];
        }
    }

    /// Copies the stack values starting at the specified position at the specified clock cycle to
    /// position - 1 at the next clock cycle.
    ///
    /// The final register is filled with the provided value in `last_value`.
    pub fn stack_shift_left_at(&mut self, clk: usize, start_pos: usize, last_value: Felt) {
        for i in start_pos..=MAX_TOP_IDX {
            self.stack[i - 1][clk + 1] = self.stack[i][clk];
        }
        self.stack[MAX_TOP_IDX][clk + 1] = last_value;
    }

    /// Copies stack values starting at the specified position at the specified clock cycle to
    /// position + 1 at the next clock cycle.
    pub fn stack_shift_right_at(&mut self, clk: usize, start_pos: usize) {
        for i in start_pos..MAX_TOP_IDX {
            self.stack[i + 1][clk + 1] = self.stack[i][clk];
        }
    }

    // BOOKKEEPING & HELPER COLUMN ACCESSORS AND MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Returns the trace state of the stack helper columns at the specified clock cycle.
    #[allow(dead_code)]
    pub fn get_helpers_state_at(&self, clk: usize) -> [Felt; NUM_STACK_HELPER_COLS] {
        let mut result = [Felt::ZERO; NUM_STACK_HELPER_COLS];
        for (result, column) in result.iter_mut().zip(self.helpers.iter()) {
            *result = column[clk];
        }
        result
    }

    /// Copies the helper values at the specified clock cycle to the next clock cycle.
    pub fn copy_helpers_at(&mut self, clk: usize) {
        for i in 0..NUM_STACK_HELPER_COLS {
            self.helpers[i][clk + 1] = self.helpers[i][clk];
        }
    }

    /// Updates the bookkeeping and helper columns to manage a right shift at the specified clock
    /// cycle.
    ///
    /// This function assumes that the stack depth has been increased by one and a new row has been
    /// added to the overflow table. It makes the following changes to the helper columns.
    ///
    /// b0: Increment the stack depth by one.
    /// b1: Save the address of the new top row in overflow table, which is the current clock cycle.
    /// h0: Set the value to 1 / (depth - 16).
    pub fn helpers_shift_right_at(&mut self, clk: usize) {
        // Increment b0 by one.
        let b0 = self.helpers[0][clk] + Felt::ONE;
        self.helpers[0][clk + 1] = b0;
        // Set b1 to the curren tclock cycle.
        self.helpers[1][clk + 1] = Felt::new(clk as u64);
        // Update the helper column to 1 / (b0 - 16).
        self.helpers[2][clk + 1] = Felt::ONE / (b0 - Felt::new(MIN_STACK_DEPTH as u64));
    }

    /// Updates the bookkeeping and helper columns to manage a left shift at the specified clock
    /// cycle.
    ///
    /// This function assumes that the stack depth has been decreased by one and a row has been
    /// removed from the overflow table. It makes the following changes to the helper columns.
    ///
    /// b0: Decrement the stack depth by one.
    /// b1: Update the address of the top row in the overflow table to the specified
    /// `next_overflow_addr`.
    /// h0: Set the value to 1 / (depth - 16) if the depth is still greater than the minimum stack
    /// depth, or to zero otherwise.
    pub fn helpers_shift_left_at(&mut self, clk: usize, next_overflow_addr: Felt) {
        // Decrement b0 by one.
        let b0 = self.helpers[0][clk] - Felt::ONE;
        self.helpers[0][clk + 1] = b0;

        // Set b1 to the overflow table address of the item at the top of the updated table.
        self.helpers[1][clk + 1] = next_overflow_addr;

        // Update the helper column to 1 / (b0 - 16) if depth > MIN_STACK_DEPTH or 0 otherwise.
        let h0 = if b0.as_int() > MIN_STACK_DEPTH as u64 {
            Felt::ONE / (b0 - Felt::new(MIN_STACK_DEPTH as u64))
        } else {
            Felt::ZERO
        };
        self.helpers[2][clk + 1] = h0;
    }

    // UTILITY METHODS
    // --------------------------------------------------------------------------------------------

    /// Makes sure there is enough memory allocated for the trace to accommodate a new row.
    ///
    /// Trace length is doubled every time it needs to be increased.
    pub fn ensure_trace_capacity(&mut self, clk: usize) {
        let current_capacity = get_trace_len(&self.stack);
        if clk + 1 >= current_capacity {
            let new_length = current_capacity * 2;
            for register in self.stack.iter_mut().chain(self.helpers.iter_mut()) {
                register.resize(new_length, Felt::ZERO);
            }
        }
    }
}
