use super::{
    enforce_constraints, Hasher, HASHER_STATE_COL_RANGE, NODE_INDEX_COL_IDX, NUM_CONSTRAINTS,
    ROW_COL_IDX, SELECTOR_COL_RANGE,
};
use rand_utils::rand_array;
use vm_core::{
    hasher::{apply_round, Selectors, LINEAR_HASH, STATE_WIDTH},
    Felt, FieldElement, TRACE_WIDTH,
};
use winter_air::EvaluationFrame;

// UNIT TESTS
// ================================================================================================

/// Tests instruction RPR, which is executed on all cycles that are not one less than a multiple of
/// eight, and applies a round of Rescue-XLIX.
#[test]
fn rescue_round() {
    let expected = [Felt::ZERO; NUM_CONSTRAINTS];

    let cycle_row_num: usize = 3;
    let current_selectors = [Felt::ZERO, LINEAR_HASH[1], LINEAR_HASH[2]];
    let next_selectors = current_selectors;

    let frame = get_test_hashing_frame(current_selectors, next_selectors, cycle_row_num);
    let result = get_constraint_evaluation(frame, cycle_row_num);
    assert_eq!(expected, result);
}

// TEST HELPER FUNCTIONS
// ================================================================================================

/// Returns the result of Hash processor's constraint evaluations on the provided frame starting at
/// the specified row.
fn get_constraint_evaluation(
    frame: EvaluationFrame<Felt>,
    cycle_row_num: usize,
) -> [Felt; NUM_CONSTRAINTS] {
    let mut result = [Felt::ZERO; NUM_CONSTRAINTS];
    let periodic_values = get_test_periodic_values(cycle_row_num);

    enforce_constraints(&frame, &periodic_values, &mut result, Felt::ONE, Felt::ONE);

    result
}

/// Returns the values from the periodic columns for the specified cycle row.
fn get_test_periodic_values(cycle_row: usize) -> Vec<Felt> {
    // Set the periodic column values.
    let mut periodic_values = match cycle_row {
        0 => vec![Felt::ZERO, Felt::ZERO, Felt::ONE],
        7 => vec![Felt::ZERO, Felt::ONE, Felt::ZERO],
        8 => vec![Felt::ONE, Felt::ZERO, Felt::ZERO],
        _ => vec![Felt::ZERO, Felt::ZERO, Felt::ZERO],
    };

    // Add the Rescue Prime round constants for the first 7 rows of the cycle, or pad with zeros.
    if cycle_row == 7 {
        periodic_values.resize(periodic_values.len() + STATE_WIDTH * 2, Felt::ZERO);
    } else {
        periodic_values.extend_from_slice(&Hasher::ARK1[cycle_row]);
        periodic_values.extend_from_slice(&Hasher::ARK2[cycle_row]);
    }
    periodic_values
}

/// Returns a valid test frame for a transition where one round of Rescue-XLIX is computed.
fn get_test_hashing_frame(
    current_selectors: Selectors,
    next_selectors: Selectors,
    cycle_row_num: usize,
) -> EvaluationFrame<Felt> {
    let mut current = vec![Felt::ZERO; TRACE_WIDTH];
    let mut next = vec![Felt::ZERO; TRACE_WIDTH];

    // Set the selectors for the hash operation.
    current[SELECTOR_COL_RANGE].copy_from_slice(&current_selectors);
    next[SELECTOR_COL_RANGE].copy_from_slice(&next_selectors);

    // add the row values
    current[ROW_COL_IDX] = Felt::new(cycle_row_num as u64);
    next[ROW_COL_IDX] = Felt::new(cycle_row_num as u64 + 1);

    // Set the starting hasher state.
    let mut state = rand_array();
    current[HASHER_STATE_COL_RANGE].copy_from_slice(&state);

    // Set the hasher state after a single permutation.
    apply_round(&mut state, cycle_row_num);
    next[HASHER_STATE_COL_RANGE].copy_from_slice(&state);

    // Set the node index values to zero for hash computations.
    current[NODE_INDEX_COL_IDX] = Felt::ZERO;
    next[NODE_INDEX_COL_IDX] = Felt::ZERO;

    EvaluationFrame::from_rows(current, next)
}
