use crate::{utils::get_trace_len, CodeBlock, ExecutionTrace, Operation, Process};
use vm_core::{
    chiplets::bitwise::{BITWISE_OR, OP_CYCLE_LEN},
    hasher::{HASH_CYCLE_LEN, LINEAR_HASH, RETURN_STATE},
    Felt, FieldElement, ProgramInputs, CHIPLETS_RANGE, CHIPLETS_WIDTH,
};

type ChipletsTrace = [Vec<Felt>; CHIPLETS_WIDTH];

#[test]
fn hasher_aux_trace() {
    // --- single hasher permutation with no stack manipulation -----------------------------------
    let stack = [2, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0];
    let operations = vec![Operation::RpPerm];
    let (chiplets_trace, trace_len) = build_trace(&stack, operations);

    // Skip the hash of the span block generated while building the trace to check only the RpPerm.
    let hasher_start = HASH_CYCLE_LEN;
    let hasher_end = hasher_start + HASH_CYCLE_LEN;
    validate_hasher_trace(&chiplets_trace, hasher_start, hasher_end);

    // Validate that the trace was padded correctly.
    validate_padding(&chiplets_trace, hasher_end, trace_len);
}

#[test]
fn bitwise_aux_trace() {
    // --- single bitwise operation with no stack manipulation ------------------------------------
    let stack = [4, 8];
    let operations = vec![Operation::U32or];
    let (chiplets_trace, trace_len) = build_trace(&stack, operations);

    let bitwise_end = HASH_CYCLE_LEN + OP_CYCLE_LEN;
    validate_bitwise_trace(&chiplets_trace, HASH_CYCLE_LEN, bitwise_end);

    // Validate that the trace was padded correctly.
    validate_padding(&chiplets_trace, bitwise_end, trace_len - 1);
}

#[test]
fn memory_aux_trace() {
    // --- single memory operation with no stack manipulation -------------------------------------
    let stack = [1, 2, 3, 4];
    let operations = vec![Operation::Push(Felt::new(2)), Operation::MStoreW];
    let (chiplets_trace, trace_len) = build_trace(&stack, operations);
    let memory_trace_len = 1;

    // Skip the hash cycle created by the span block when building the trace.
    // Check the memory trace.
    let memory_end = HASH_CYCLE_LEN + memory_trace_len;
    validate_memory_trace(&chiplets_trace, HASH_CYCLE_LEN, memory_end, 2);

    // Validate that the trace was padded correctly.
    validate_padding(&chiplets_trace, memory_end, trace_len);
}

#[test]
fn stacked_aux_trace() {
    // --- operations in hasher, bitwise, and memory processors without stack manipulation --------
    let stack = [8, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 1];
    let operations = vec![
        Operation::U32or,
        Operation::Push(Felt::ZERO),
        Operation::MStoreW,
        Operation::RpPerm,
    ];
    let (chiplets_trace, trace_len) = build_trace(&stack, operations);
    let memory_len = 1;

    // Skip the hash of the span block generated while building the trace to check only the RpPerm.
    let hasher_start = HASH_CYCLE_LEN;
    let hasher_end = hasher_start + HASH_CYCLE_LEN;
    validate_hasher_trace(&chiplets_trace, hasher_start, hasher_end);

    // Expect 1 operation cycle in the bitwise trace
    let bitwise_end = hasher_end + OP_CYCLE_LEN;
    validate_bitwise_trace(&chiplets_trace, hasher_end, bitwise_end);

    // expect 1 row of memory trace
    let memory_end = bitwise_end + memory_len;
    validate_memory_trace(&chiplets_trace, bitwise_end, memory_end, 0);

    // Validate that the trace was padded correctly.
    validate_padding(&chiplets_trace, memory_end, trace_len);
}

// HELPER FUNCTIONS
// ================================================================================================

/// Builds a sample trace by executing a span block containing the specified operations. This
/// results in 1 additional hash cycle (8 rows) at the beginning of the hash chiplet.
fn build_trace(stack: &[u64], operations: Vec<Operation>) -> (ChipletsTrace, usize) {
    let inputs = ProgramInputs::new(stack, &[], vec![]).unwrap();
    let mut process = Process::new(inputs);
    let program = CodeBlock::new_span(operations);
    process.execute_code_block(&program).unwrap();

    let (trace, _) = ExecutionTrace::test_finalize_trace(process);
    let trace_len = get_trace_len(&trace) - ExecutionTrace::NUM_RAND_ROWS;

    (
        trace[CHIPLETS_RANGE]
            .to_vec()
            .try_into()
            .expect("failed to convert vector to array"),
        trace_len,
    )
}

/// Validate the hasher trace output by the rpperm operation. The full hasher trace is tested in
/// the Hasher module, so this just tests the ChipletsTrace selectors and the initial columns
/// of the hasher trace.
fn validate_hasher_trace(chiplets: &ChipletsTrace, start: usize, end: usize) {
    // The selectors should match the hasher selectors
    for row in start..end {
        // The selectors should match the selectors for the hasher segment
        assert_eq!(Felt::ZERO, chiplets[0][row]);

        match row % HASH_CYCLE_LEN {
            0 => {
                // in the first row, the expected start of the trace should hold the initial selectors
                assert_eq!(
                    LINEAR_HASH,
                    [chiplets[1][row], chiplets[2][row], chiplets[3][row]]
                );
            }
            7 => {
                // in the last row, the expected start of the trace should hold the final selectors
                assert_eq!(
                    RETURN_STATE,
                    [chiplets[1][row], chiplets[2][row], chiplets[3][row]]
                );
            }
            _ => {
                // in the other rows, the expected start of the trace should hold the mid selectors
                assert_eq!(
                    [Felt::ZERO, LINEAR_HASH[1], LINEAR_HASH[2]],
                    [chiplets[1][row], chiplets[2][row], chiplets[3][row]]
                );
            }
        }
    }
}

/// Validate the bitwise trace output by the u32or operation. The full bitwise trace is tested in
/// the Bitwise module, so this just tests the ChipletsTrace selectors, the initial columns
/// of the bitwise trace, and the final columns after the bitwise trace.
fn validate_bitwise_trace(chiplets: &ChipletsTrace, start: usize, end: usize) {
    // The selectors should match the bitwise selectors
    for row in start..end {
        // The selectors should match the selectors for the bitwise segment
        assert_eq!(Felt::ONE, chiplets[0][row]);
        assert_eq!(Felt::ZERO, chiplets[1][row]);

        // the expected start of the bitwise trace should hold the expected bitwise op selectors
        assert_eq!(BITWISE_OR, [chiplets[2][row], chiplets[3][row]]);

        // the final columns should be padded
        assert_eq!(Felt::ZERO, chiplets[16][row]);
        assert_eq!(Felt::ZERO, chiplets[17][row]);
    }
}

/// Validate the bitwise trace output by the storew operation. The full memory trace is tested in
/// the Memory module, so this just tests the ChipletsTrace selectors, the initial columns
/// of the memory trace, and the final column after the memory trace.
fn validate_memory_trace(chiplets: &ChipletsTrace, start: usize, end: usize, addr: u64) {
    for row in start..end {
        // The selectors in the first row should match the memory selectors
        assert_eq!(Felt::ONE, chiplets[0][row]);
        assert_eq!(Felt::ONE, chiplets[1][row]);
        assert_eq!(Felt::ZERO, chiplets[2][row]);

        // the expected start of the memory trace should hold the memory ctx and addr
        assert_eq!(Felt::ZERO, chiplets[3][row]);
        assert_eq!(Felt::new(addr), chiplets[4][row]);

        // the final column should be padded
        assert_eq!(Felt::ZERO, chiplets[17][row]);
    }
}

/// Checks that the final section of the chiplets module's trace after the memory chiplet is
/// padded and has the correct selectors.
fn validate_padding(chiplets: &ChipletsTrace, start: usize, end: usize) {
    for row in start..end {
        // selectors
        assert_eq!(Felt::ONE, chiplets[0][row]);
        assert_eq!(Felt::ONE, chiplets[1][row]);
        assert_eq!(Felt::ONE, chiplets[2][row]);

        // padding
        chiplets.iter().skip(3).for_each(|column| {
            assert_eq!(Felt::ZERO, column[row]);
        });
    }
}
