use crate::build_op_test;
use crate::helpers::{prop_randw, TestError, WORD_LEN};

mod arithmetic_ops;
mod bitwise_ops;
mod comparison_ops;
mod conversion_ops;

// CONSTANTS
// ================================================================================================
pub const U32_BOUND: u64 = u32::MAX as u64 + 1;

// HELPER FUNCTIONS
// ================================================================================================

/// This helper function tests a provided u32 assembly operation, which takes a single input, to
/// ensure that it fails when the input is >= 2^32.
pub fn test_input_out_of_bounds(asm_op: &str) {
    let test = build_op_test!(asm_op, &[U32_BOUND]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));
}

/// This helper function tests a provided u32 assembly operation, which takes multiple inputs, to
/// ensure that it fails when any one of the inputs is >= 2^32. Each input is tested independently.
pub fn test_inputs_out_of_bounds(asm_op: &str, input_count: usize) {
    let inputs = vec![0_u64; input_count];

    for i in 0..input_count {
        let mut i_inputs = inputs.clone();
        // should fail when the value of the input at index i is out of bounds
        i_inputs[i] = U32_BOUND;

        let test = build_op_test!(asm_op, &i_inputs);
        test.expect_error(TestError::ExecutionError("NotU32Value"));
    }
}

/// This helper function tests a provided assembly operation which takes a single parameter
/// to ensure that it fails when that parameter is over the maximum allowed value (out of bounds).
pub fn test_param_out_of_bounds(asm_op_base: &str, gt_max_value: u64) {
    let asm_op = format!("{}.{}", asm_op_base, gt_max_value);
    let test = build_op_test!(&asm_op);
    test.expect_error(TestError::AssemblyError("parameter"));
}

/// This helper function tests that when the given u32 assembly instruction is executed on
/// out-of-bounds inputs it does not fail. Each input is tested independently.
pub fn test_unchecked_execution(asm_op: &str, input_count: usize) {
    let values = vec![1_u64; input_count];

    for i in 0..input_count {
        let mut i_values = values.clone();
        // should execute successfully when the value of the input at index i is out of bounds
        i_values[i] = U32_BOUND;

        let test = build_op_test!(asm_op, &i_values);
        assert!(test.execute().is_ok());
    }
}
