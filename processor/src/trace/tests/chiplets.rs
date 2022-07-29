use super::{
    super::{Trace, NUM_RAND_ROWS},
    build_trace_from_ops, rand_array, ExecutionTrace, Felt, FieldElement, Operation, Word, ONE,
    ZERO,
};
use rand_utils::rand_value;
use vm_core::{
    chiplets::{
        bitwise::{
            Selectors, BITWISE_AND, BITWISE_AND_LABEL, BITWISE_OR, BITWISE_OR_LABEL, BITWISE_XOR,
            BITWISE_XOR_LABEL, OP_CYCLE_LEN,
        },
        hasher::HASH_CYCLE_LEN,
        memory::{
            ADDR_COL_IDX, CLK_COL_IDX, CTX_COL_IDX, MEMORY_LABEL, NUM_ELEMENTS, U_COL_RANGE,
            V_COL_RANGE,
        },
        BITWISE_A_COL_IDX, BITWISE_B_COL_IDX, BITWISE_OUTPUT_COL_IDX, BITWISE_TRACE_OFFSET,
    },
    AUX_TRACE_RAND_ELEMENTS, CHIPLETS_AUX_TRACE_OFFSET,
};

/// Tests the generation of the `b_aux` bus column when only bitwise lookups are included. It
/// ensures that trace generation is correct when all of the following are true.
///
/// - All possible bitwise operations are called by the stack.
/// - Some requests from the Stack and responses from the Bitwise chiplet occur at the same cycle.
#[test]
#[allow(clippy::needless_range_loop)]
fn b_aux_trace_bitwise() {
    let a = rand_value::<u32>();
    let b = rand_value::<u32>();
    let stack = [a as u64, b as u64];
    let operations = vec![
        Operation::U32and,
        Operation::Push(Felt::from(a)),
        Operation::Push(Felt::from(b)),
        Operation::U32or,
        // Add 8 padding operations so that U32xor is requested by the stack in the same cycle when
        // U32and is provided by the Bitwise chiplet.
        Operation::Pad,
        Operation::Pad,
        Operation::Pad,
        Operation::Pad,
        Operation::Drop,
        Operation::Drop,
        Operation::Drop,
        Operation::Drop,
        Operation::Push(Felt::from(a)),
        Operation::Push(Felt::from(b)),
        Operation::U32xor,
        // Drop 4 values to empty the stack's overflow table.
        Operation::Drop,
        Operation::Drop,
        Operation::Drop,
        Operation::Drop,
    ];
    let mut trace = build_trace_from_ops(operations, &stack);

    let rand_elements = rand_array::<Felt, AUX_TRACE_RAND_ELEMENTS>();
    let aux_columns = trace.build_aux_segment(&[], &rand_elements).unwrap();
    let b_aux = aux_columns.get_column(CHIPLETS_AUX_TRACE_OFFSET);

    assert_eq!(trace.length(), b_aux.len());
    assert_eq!(ONE, b_aux[0]);
    assert_eq!(ONE, b_aux[1]);

    // The first bitwise request from the stack is sent when the `U32and` operation is executed at
    // cycle 1, so the request is included in the next row. (The trace begins by executing `span`).
    let value = build_expected_bitwise(
        &rand_elements,
        BITWISE_AND_LABEL,
        Felt::from(a),
        Felt::from(b),
        Felt::from(a & b),
    );
    let mut expected = value.inv();
    assert_eq!(expected, b_aux[2]);

    // Nothing changes during user operations with no requests to the Chiplets.
    for row in 3..5 {
        assert_eq!(expected, b_aux[row]);
    }

    // The second bitwise request from the stack is sent when the `U32or` operation is executed at
    // cycle 4, so the request is included in the next row.
    let value = build_expected_bitwise(
        &rand_elements,
        BITWISE_OR_LABEL,
        Felt::from(a),
        Felt::from(b),
        Felt::from(a | b),
    );
    expected *= value.inv();
    assert_eq!(expected, b_aux[5]);

    // Nothing changes during user operations with no requests to the Chiplets.
    for row in 6..16 {
        assert_eq!(expected, b_aux[row]);
    }

    // Bitwise responses will be provided during the bitwise segment of the Chiplets trace,
    // which starts after the hash for the span block. Responses are provided at the last row of the
    // Bitwise chiplet's operation cycle.
    let response_1_row = HASH_CYCLE_LEN + OP_CYCLE_LEN;
    let response_2_row = response_1_row + OP_CYCLE_LEN;
    let response_3_row = response_2_row + OP_CYCLE_LEN;

    // At cycle 15, `U32xor` is requested by the stack and `U32and` is provided by the bitwise
    // chiplet.
    let value = build_expected_bitwise(
        &rand_elements,
        BITWISE_XOR_LABEL,
        Felt::from(a),
        Felt::from(b),
        Felt::from(a ^ b),
    );
    expected *= value.inv();
    expected *= build_expected_bitwise_from_trace(&trace, &rand_elements, response_1_row - 1);
    assert_eq!(expected, b_aux[response_1_row]);

    // Nothing changes until the next time the Bitwise chiplet responds.
    for row in response_1_row..response_2_row {
        assert_eq!(expected, b_aux[row]);
    }

    // At the end of the next bitwise cycle, the response for `U32or` is provided by the Bitwise
    // chiplet.
    expected *= build_expected_bitwise_from_trace(&trace, &rand_elements, response_2_row - 1);
    assert_eq!(expected, b_aux[response_2_row]);

    // Nothing changes until the next time the Bitwise chiplet responds.
    for row in response_2_row..response_3_row {
        assert_eq!(expected, b_aux[row]);
    }

    // At the end of the next bitwise cycle, the response for `U32or` is provided by the Bitwise
    // chiplet.
    expected *= build_expected_bitwise_from_trace(&trace, &rand_elements, response_3_row - 1);
    assert_eq!(expected, b_aux[response_3_row]);

    // The value in b_aux should be ONE now and for the rest of the trace.
    for row in response_3_row..trace.length() - NUM_RAND_ROWS {
        assert_eq!(ONE, b_aux[row]);
    }
}

/// Tests the generation of the `b_aux` bus column when only memory lookups are included. It ensures
/// that trace generation is correct when all of the following are true.
///
/// - All possible memory operations are called by the stack.
/// - Some requests from the Stack and responses from Memory occur at the same cycle.
/// - Multiple memory addresses are used.
#[test]
#[allow(clippy::needless_range_loop)]
fn b_aux_trace_mem() {
    let stack = [1, 2, 3, 4, 0];
    let word = [ONE, Felt::new(2), Felt::new(3), Felt::new(4)];
    let operations = vec![
        Operation::MStoreW, // store [1, 2, 3, 4]
        Operation::Drop,    // clear the stack
        Operation::Drop,
        Operation::Drop,
        Operation::Drop,
        Operation::MLoad,     // read the first value of the word
        Operation::MovDn5,    // put address 0 and space for a full word at top of stack
        Operation::MLoadW,    // load word from address 0 to stack
        Operation::Push(ONE), // push a new value onto the stack
        Operation::Push(ONE), // push a new address on to the stack
        Operation::MStore,    // store 1 at address 1
        Operation::Drop,      // ensure the stack overflow table is empty
    ];
    let mut trace = build_trace_from_ops(operations, &stack);

    let rand_elements = rand_array::<Felt, AUX_TRACE_RAND_ELEMENTS>();
    let aux_columns = trace.build_aux_segment(&[], &rand_elements).unwrap();
    let b_aux = aux_columns.get_column(CHIPLETS_AUX_TRACE_OFFSET);

    assert_eq!(trace.length(), b_aux.len());
    assert_eq!(ONE, b_aux[0]);
    assert_eq!(ONE, b_aux[1]);

    // The first memory request from the stack is sent when the `MStoreW` operation is executed, at
    // cycle 1, so the request is included in the next row. (The trace begins by executing `span`).
    let value = build_expected_memory(&rand_elements, ZERO, ZERO, Felt::new(1), [ZERO; 4], word);
    let mut expected = value.inv();
    assert_eq!(expected, b_aux[2]);

    // Nothing changes after user operations that don't make requests to the Chiplets.
    for row in 3..7 {
        assert_eq!(expected, b_aux[row]);
    }

    // The next memory request from the stack is sent when `MLoad` is executed at cycle 6 and
    // included at row 7
    let value = build_expected_memory(&rand_elements, ZERO, ZERO, Felt::new(6), word, word);
    expected *= value.inv();
    assert_eq!(expected, b_aux[7]);

    // Nothing changes in row 8.
    assert_eq!(expected, b_aux[8]);

    // Memory responses will be provided during the memory segment of the Chiplets trace,
    // which starts after the hash for the span block at row 8. There will be 4 rows, corresponding
    // to the four Memory operations.

    // At cycle 8 `MLoadW` is requested by the stack and `MStoreW` is provided by memory
    let value = build_expected_memory(&rand_elements, ZERO, ZERO, Felt::new(8), word, word);
    expected *= value.inv();
    expected *= build_expected_memory_from_trace(&trace, &rand_elements, 8);
    assert_eq!(expected, b_aux[9]);

    // At cycle 9, `MLoad` is provided by memory.
    expected *= build_expected_memory_from_trace(&trace, &rand_elements, 9);
    assert_eq!(expected, b_aux[10]);

    // At cycle 10,  `MLoadW` is provided by memory.
    expected *= build_expected_memory_from_trace(&trace, &rand_elements, 10);
    assert_eq!(expected, b_aux[11]);

    // At cycle 11, `MStore` is requested by the stack and provided by memory.
    let value = build_expected_memory(
        &rand_elements,
        ZERO,
        ONE,
        Felt::new(11),
        [ZERO; 4],
        [ONE, ZERO, ZERO, ZERO],
    );
    expected *= value.inv();
    expected *= build_expected_memory_from_trace(&trace, &rand_elements, 11);
    assert_eq!(expected, b_aux[12]);

    // The value in b_aux should be ONE now and for the rest of the trace.
    for row in 12..trace.length() - NUM_RAND_ROWS {
        assert_eq!(ONE, b_aux[row]);
    }
}

// TEST HELPERS
// ================================================================================================

fn build_expected_bitwise(alphas: &[Felt], op_id: Felt, a: Felt, b: Felt, result: Felt) -> Felt {
    alphas[0] + alphas[1] * op_id + alphas[2] * a + alphas[3] * b + alphas[4] * result
}

fn build_expected_bitwise_from_trace(trace: &ExecutionTrace, alphas: &[Felt], row: usize) -> Felt {
    let s0 = trace.main_trace.get_column(BITWISE_TRACE_OFFSET)[row];
    let s1 = trace.main_trace.get_column(BITWISE_TRACE_OFFSET + 1)[row];
    let selectors: Selectors = [s0, s1];

    let op_id = if selectors == BITWISE_AND {
        BITWISE_AND_LABEL
    } else if selectors == BITWISE_OR {
        BITWISE_OR_LABEL
    } else if selectors == BITWISE_XOR {
        BITWISE_XOR_LABEL
    } else {
        panic!("Execution trace contains an invalid bitwise operation.")
    };

    let a = trace.main_trace.get_column(BITWISE_A_COL_IDX)[row];
    let b = trace.main_trace.get_column(BITWISE_B_COL_IDX)[row];
    let output = trace.main_trace.get_column(BITWISE_OUTPUT_COL_IDX)[row];

    build_expected_bitwise(alphas, op_id, a, b, output)
}

fn build_expected_memory(
    alphas: &[Felt],
    ctx: Felt,
    addr: Felt,
    clk: Felt,
    old_word: Word,
    new_word: Word,
) -> Felt {
    let mut old_word_value = ZERO;
    let mut new_word_value = ZERO;

    for i in 0..NUM_ELEMENTS {
        old_word_value += alphas[i + 5] * old_word[i];
        new_word_value += alphas[i + 9] * new_word[i];
    }

    alphas[0]
        + alphas[1] * MEMORY_LABEL
        + alphas[2] * ctx
        + alphas[3] * addr
        + alphas[4] * clk
        + old_word_value
        + new_word_value
}

fn build_expected_memory_from_trace(trace: &ExecutionTrace, alphas: &[Felt], row: usize) -> Felt {
    let ctx = trace.main_trace.get_column(CTX_COL_IDX)[row];
    let addr = trace.main_trace.get_column(ADDR_COL_IDX)[row];
    let clk = trace.main_trace.get_column(CLK_COL_IDX)[row];
    let mut old_word = [ZERO; NUM_ELEMENTS];
    let mut new_word = [ZERO; NUM_ELEMENTS];

    for i in 0..NUM_ELEMENTS {
        old_word[i] = trace.main_trace.get_column(U_COL_RANGE.start + i)[row];
        new_word[i] = trace.main_trace.get_column(V_COL_RANGE.start + i)[row];
    }

    build_expected_memory(alphas, ctx, addr, clk, old_word, new_word)
}
