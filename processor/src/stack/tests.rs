use vm_core::{FieldElement, NUM_STACK_HELPER_COLS};

use super::{Felt, ProgramInputs, Stack, StackTopState, MIN_STACK_DEPTH};

#[test]
fn initialize() {
    // initialize a new stack with some initial values
    let mut stack_inputs = [1, 2, 3, 4];
    let inputs = ProgramInputs::new(&stack_inputs, &[], vec![]).unwrap();
    let stack = Stack::new(&inputs, 4, false);

    // Prepare the expected results.
    stack_inputs.reverse();
    let expected_stack = build_stack(&[4, 3, 2, 1]);
    let expected_helpers = [Felt::new(MIN_STACK_DEPTH as u64), Felt::ZERO, Felt::ZERO];

    // Check the stack state.
    assert_eq!(stack.trace_state(), expected_stack);

    // Check the helper columns.
    assert_eq!(
        stack.trace.get_helpers_state_at(stack.current_clk()),
        expected_helpers
    );
}

#[test]
fn shift_left() {
    let inputs = ProgramInputs::new(&[1, 2, 3, 4], &[], vec![]).unwrap();
    let mut stack = Stack::new(&inputs, 4, false);

    // ---- left shift an entire stack of minimum depth -------------------------------------------
    // Prepare the expected results.
    let expected_stack = build_stack(&[3, 2, 1]);
    let expected_helpers = build_helpers_left(0, 0);

    // Perform the left shift.
    stack.shift_left(1);
    stack.advance_clock();

    // Check the stack state.
    assert_eq!(stack.trace_state(), expected_stack);

    // Check the helper columns.
    assert_eq!(
        stack.trace.get_helpers_state_at(stack.current_clk()),
        expected_helpers
    );

    // ---- left shift an entire stack with multiple overflow items -------------------------------
    let mut stack = Stack::new(&inputs, 4, false);
    // Shift right twice to add 2 items to the overflow table.
    stack.shift_right(0);
    let prev_overflow_addr = stack.current_clk();
    stack.advance_clock();
    stack.shift_right(0);
    stack.advance_clock();

    // Prepare the expected results.
    let expected_stack = build_stack(&[0, 4, 3, 2, 1]);
    let expected_helpers = build_helpers_left(1, prev_overflow_addr);

    // Perform the left shift.
    stack.shift_left(1);
    stack.advance_clock();

    // Check the stack state.
    assert_eq!(stack.trace_state(), expected_stack);

    // Check the helper columns.
    assert_eq!(
        stack.trace.get_helpers_state_at(stack.current_clk()),
        expected_helpers
    );

    // ---- left shift an entire stack with one overflow item -------------------------------------
    // Prepare the expected results.
    let expected_stack = build_stack(&[4, 3, 2, 1]);
    let expected_helpers = build_helpers_left(0, 0);

    // Perform the left shift.
    stack.ensure_trace_capacity();
    stack.shift_left(1);
    stack.advance_clock();

    // Check the stack state.
    assert_eq!(stack.trace_state(), expected_stack);

    // Check the helper columns.
    assert_eq!(
        stack.trace.get_helpers_state_at(stack.current_clk()),
        expected_helpers
    );
}

#[test]
fn shift_right() {
    let inputs = ProgramInputs::new(&[1, 2, 3, 4], &[], vec![]).unwrap();
    let mut stack = Stack::new(&inputs, 4, false);

    // ---- right shift an entire stack of minimum depth ------------------------------------------
    let expected_stack = build_stack(&[0, 4, 3, 2, 1]);
    let expected_helpers = build_helpers_right(1, stack.current_clk());

    stack.shift_right(0);
    stack.advance_clock();

    // Check the stack state.
    assert_eq!(stack.trace_state(), expected_stack);

    // Check the helper columns.
    assert_eq!(
        stack.trace.get_helpers_state_at(stack.current_clk()),
        expected_helpers
    );

    // ---- right shift when the overflow table is non-empty --------------------------------------
    let expected_stack = build_stack(&[0, 0, 4, 3, 2, 1]);
    let expected_helpers = build_helpers_right(2, stack.current_clk());

    stack.shift_right(0);
    stack.advance_clock();

    // Check the stack state.
    assert_eq!(stack.trace_state(), expected_stack);

    // Check the helper columns.
    assert_eq!(
        stack.trace.get_helpers_state_at(stack.current_clk()),
        expected_helpers
    );
}

// HELPERS
// ================================================================================================

/// Builds the trace row of stack helpers expected as the result of a right shift at clock cycle
/// `clk` when there are `num_overflow` items in the overflow table.
fn build_helpers_right(num_overflow: usize, clk: usize) -> [Felt; NUM_STACK_HELPER_COLS] {
    let b0 = Felt::new((MIN_STACK_DEPTH + num_overflow) as u64);
    let b1 = Felt::new(clk as u64);
    let h0 = Felt::ONE / (b0 - Felt::new(MIN_STACK_DEPTH as u64));

    [b0, b1, h0]
}

/// Builds the trace row of stack helpers expected as the result of a left shift when there are
/// `num_overflow` items in the overflow table and the top row in the table has address
/// `next_overflow_addr`.
fn build_helpers_left(
    num_overflow: usize,
    next_overflow_addr: usize,
) -> [Felt; NUM_STACK_HELPER_COLS] {
    let depth = MIN_STACK_DEPTH + num_overflow;
    let b0 = Felt::new(depth as u64);
    let b1 = Felt::new(next_overflow_addr as u64);
    let h0 = if depth > MIN_STACK_DEPTH {
        Felt::ONE / (b0 - Felt::new(MIN_STACK_DEPTH as u64))
    } else {
        Felt::ZERO
    };

    [b0, b1, h0]
}

/// Builds a [StackTopState] that starts with the provided stack inputs and is padded with zeros
/// until the minimum stack depth.
fn build_stack(stack_inputs: &[u64]) -> StackTopState {
    let mut expected_stack = [Felt::ZERO; MIN_STACK_DEPTH];
    for (idx, &input) in stack_inputs.iter().enumerate() {
        expected_stack[idx] = Felt::new(input);
    }

    expected_stack
}
