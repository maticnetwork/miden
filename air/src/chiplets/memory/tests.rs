use super::{
    EvaluationFrame, ADDR_COL_IDX, CLK_COL_IDX, CTX_COL_IDX, D0_COL_IDX, D1_COL_IDX, D_INV_COL_IDX,
    NUM_ELEMENTS, U_COL_RANGE, V_COL_RANGE,
};
use crate::{chiplets::memory, Felt, FieldElement};
use vm_core::TRACE_WIDTH;

use rand_utils::rand_value;

// UNIT TESTS
// ================================================================================================

#[test]
fn test_memory_write() {
    let expected = [Felt::ZERO; memory::NUM_CONSTRAINTS];

    let old_values = vec![0, 0, 0, 0];
    let new_values = vec![1, 0, 0, 0];

    // Write to a new context.
    let result = get_constraint_evaluation(MemoryTestDeltaType::Context, &old_values, &new_values);
    assert_eq!(expected, result);

    // Write to a new address in the same context.
    let result = get_constraint_evaluation(MemoryTestDeltaType::Address, &old_values, &new_values);
    assert_eq!(expected, result);

    // Write to the same context and address at a new clock cycle.
    let result = get_constraint_evaluation(MemoryTestDeltaType::Clock, &old_values, &new_values);
    assert_eq!(expected, result);
}

#[test]
fn test_memory_read() {
    let expected = [Felt::ZERO; memory::NUM_CONSTRAINTS];

    let old_values = vec![1, 0, 0, 0];
    let new_values = vec![1, 0, 0, 0];

    // Memory reads only happen when neither the context nor the address have changed.
    let result = get_constraint_evaluation(MemoryTestDeltaType::Clock, &old_values, &new_values);
    assert_eq!(expected, result);
}

// TEST HELPERS
// ================================================================================================

/// Specifies the column where the delta should occur in a memory test frame.
/// - Context: when the delta is in context, the address and clock columns can also change.
/// - Address: when the delta occurs in the address, context must stay fixed, but clock can change.
/// - Clock: when the delta occurs in the clock column, context and address must stay fixed.
enum MemoryTestDeltaType {
    Context,
    Address,
    Clock,
}

/// Generates a frame that reads or writes memory with the specified old and new values and a change
/// in the  specified column (context, address, or clock), then returns the evaluation of the memory
/// constraints on this frame.
///
/// - To test a valid write, the MemoryTestDeltaType must be Context or Address and the `old_values` and
/// `new_values` must change.
/// - To test a valid read, the `delta_type` must be Clock and the `old_values` and `new_values`
/// must be equal.
fn get_constraint_evaluation(
    delta_type: MemoryTestDeltaType,
    old_values: &[u32],
    new_values: &[u32],
) -> [Felt; memory::NUM_CONSTRAINTS] {
    let delta_row = get_test_delta_row(&delta_type);
    let frame = get_test_frame(&delta_type, &delta_row, old_values, new_values);

    let mut result = [Felt::ZERO; memory::NUM_CONSTRAINTS];

    memory::enforce_constraints(&frame, &mut result, Felt::ONE);

    result
}

/// Generates an EvaluationFrame with memory trace data as specified by the inputs. The frame treats
/// the current row as the first row of a memory execution trace with context, address, clock, old
/// values, delta (d1, d0), and delta inverse set to zero. The provided inputs determine the values
/// in the next row and therefore the transition from current to next. The generated frame will be
/// valid when valid inputs are provided.
///
/// - `delta_type`: specifies the column over which the delta value should be calculated.
/// - `delta_row`: specifies the values of the context, address, and clock columns in the next row.
/// - `old_values`: specifies the old values, which are placed in the "new" value columns of the
///   current row and the "old" value columns of the next row.
/// - `new_values`: specifies the new values, which are placed in the "new" columns of the next row.
fn get_test_frame(
    delta_type: &MemoryTestDeltaType,
    delta_row: &[u64],
    old_values: &[u32],
    new_values: &[u32],
) -> EvaluationFrame<Felt> {
    let mut current = vec![Felt::ZERO; TRACE_WIDTH];
    let mut next = vec![Felt::ZERO; TRACE_WIDTH];

    // Set the context, addr, and clock columns in the next row to the values in the delta row.
    next[CTX_COL_IDX] = Felt::new(delta_row[0]);
    next[ADDR_COL_IDX] = Felt::new(delta_row[1]);
    next[CLK_COL_IDX] = Felt::new(delta_row[2]);

    // Set the old and new values.
    for idx in 0..NUM_ELEMENTS {
        let old_value = Felt::new(old_values[idx] as u64);
        // Add a write for the old values to the current row.
        current[U_COL_RANGE.start + idx] = Felt::ZERO;
        current[V_COL_RANGE.start + idx] = old_value;
        // Change the values from old to new in the next row.
        next[U_COL_RANGE.start + idx] = old_value;
        next[V_COL_RANGE.start + idx] = Felt::new(new_values[idx] as u64);
    }

    // Set the delta and delta inverse values. Treat the current row as if it's the first row.
    current[D0_COL_IDX] = Felt::ZERO;
    current[D1_COL_IDX] = Felt::ZERO;
    current[D_INV_COL_IDX] = Felt::ZERO;

    // Set the delta in the next row according to the specified delta type.
    let delta: u64 = match delta_type {
        MemoryTestDeltaType::Clock => delta_row[MemoryTestDeltaType::Clock as usize] - 1,
        MemoryTestDeltaType::Context => delta_row[MemoryTestDeltaType::Context as usize],
        MemoryTestDeltaType::Address => delta_row[MemoryTestDeltaType::Address as usize],
    };
    next[D0_COL_IDX] = Felt::new(delta as u16 as u64);
    next[D1_COL_IDX] = Felt::new(delta >> 16);
    next[D_INV_COL_IDX] = (Felt::new(delta)).inv();

    EvaluationFrame::<Felt>::from_rows(current, next)
}

/// Generates a row of valid test values for the context, address, and clock columns according to
/// the specified delta type, which determines the column over which the delta and delta inverse
/// values of the trace would be calculated.
///
/// - When the delta type is Context, the address and clock columns can be anything.
/// - When the delta type is Address, the context must remain unchanged but the clock can change.
/// - When the delta type is Clock, both the context and address columns must remain unchanged.
fn get_test_delta_row(delta_type: &MemoryTestDeltaType) -> Vec<u64> {
    let delta_value = rand_value::<u32>() as u64;
    let mut row = vec![0; 3];
    let ctx_idx = MemoryTestDeltaType::Context as usize;
    let addr_idx = MemoryTestDeltaType::Address as usize;
    let clk_idx = MemoryTestDeltaType::Clock as usize;

    // Set the context, addr, and clock columns according to the specified delta type.
    match delta_type {
        MemoryTestDeltaType::Context => {
            // Change the row value for the context.
            row[ctx_idx] = delta_value;

            // Set addr and clock in the row column to random values.
            row[addr_idx] = rand_value::<u32>() as u64;
            row[clk_idx] = rand_value::<u32>() as u64;
        }
        MemoryTestDeltaType::Address => {
            // Keep the context value the same in current and row rows (leave it as ZERO).
            // Set the row value for the address.
            row[addr_idx] = delta_value;

            // Set clock in the row column to a random value.
            row[clk_idx] = rand_value::<u32>() as u64;
        }
        MemoryTestDeltaType::Clock => {
            // Keep the context and address values the same in the current and row rows.
            // Set the current and row values for the clock.
            row[clk_idx] = delta_value;
        }
    }

    row
}
