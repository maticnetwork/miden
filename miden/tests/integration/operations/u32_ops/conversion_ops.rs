use proptest::prelude::*;
use rand_utils::rand_value;

use super::{build_op_test, prop_randw, test_inputs_out_of_bounds, TestError, U32_BOUND, WORD_LEN};
use vm_core::{Felt, StarkField};

// U32 OPERATIONS TESTS - MANUAL - CONVERSIONS AND TESTS
// ================================================================================================

#[test]
fn u32test() {
    // pushes 1 onto the stack if a < 2^32 and 0 otherwise
    let asm_op = "u32test";

    // vars to test
    let smaller = 1_u64;
    let equal = 1_u64 << 32;
    let larger = equal + 1;

    // --- a < 2^32 -------------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[smaller]);
    test.expect_stack(&[1, smaller]);

    // --- a = 2^32 -------------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[equal]);
    test.expect_stack(&[0, equal]);

    // --- a > 2^32 -------------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[larger]);
    test.expect_stack(&[0, larger]);
}

#[test]
fn u32testw() {
    let asm_op = "u32testw";

    // --- all elements in range ------------------------------------------------------------------
    let values = [1, 1, 1, 1];
    let expected = [1, 1, 1, 1, 1];

    let test = build_op_test!(asm_op, &values);
    test.expect_stack(&expected);

    // --- 1st element >= 2^32 --------------------------------------------------------------------
    let values = [U32_BOUND, 0, 0, 0];
    let expected = [0, 0, 0, 0, U32_BOUND];

    let test = build_op_test!(asm_op, &values);
    test.expect_stack(&expected);

    // --- 2nd element >= 2^32 --------------------------------------------------------------------
    let values = [0, U32_BOUND, 0, 0];
    let expected = [0, 0, 0, U32_BOUND, 0];

    let test = build_op_test!(asm_op, &values);
    test.expect_stack(&expected);

    // --- 3rd element >= 2^32 --------------------------------------------------------------------
    let values = [0, 0, U32_BOUND, 0];
    let expected = [0, 0, U32_BOUND, 0, 0];

    let test = build_op_test!(asm_op, &values);
    test.expect_stack(&expected);

    // --- 4th element >= 2^32 --------------------------------------------------------------------
    let values = [0, 0, 0, U32_BOUND];
    let expected = [0, U32_BOUND, 0, 0, 0];

    let test = build_op_test!(asm_op, &values);
    test.expect_stack(&expected);

    // --- all elements out of range --------------------------------------------------------------
    let values = [U32_BOUND, U32_BOUND, U32_BOUND, U32_BOUND];
    let expected = [0, U32_BOUND, U32_BOUND, U32_BOUND, U32_BOUND];

    let test = build_op_test!(asm_op, &values);
    test.expect_stack(&expected);
}

#[test]
fn u32assert() {
    // assertion passes and leaves the stack unchanged if a < 2^32
    let asm_op = "u32assert";
    let asm_op_1 = "u32assert.1";

    let value = 1_u64;
    let test = build_op_test!(asm_op, &[value]);
    test.expect_stack(&[value]);

    let test = build_op_test!(asm_op_1, &[value]);
    test.expect_stack(&[value]);
}

#[test]
fn u32assert_fail() {
    // assertion fails if a >= 2^32
    let asm_op = "u32assert";
    let asm_op_1 = "u32assert.1";
    let err = "NotU32Value";

    // vars to test
    let equal = 1_u64 << 32;
    let larger = equal + 1;

    // --- test when a = 2^32 ---------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[equal]);
    test.expect_error(TestError::ExecutionError(err));

    let test_1 = build_op_test!(asm_op_1, &[equal]);
    test_1.expect_error(TestError::ExecutionError(err));

    // --- test when a > 2^32 ---------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[larger]);
    test.expect_error(TestError::ExecutionError(err));

    let test_1 = build_op_test!(asm_op_1, &[larger]);
    test_1.expect_error(TestError::ExecutionError(err));
}

#[test]
fn u32assert2() {
    // assertion passes and leaves the stack unchanged if a < 2^32 and b < 2^32
    let asm_op = "u32assert.2";
    let value_a = 1_u64;
    let value_b = 2_u64;
    let test = build_op_test!(asm_op, &[value_a, value_b]);
    test.expect_stack(&[value_b, value_a]);

    let value_a = rand_value::<u32>() as u64;
    let value_b = rand_value::<u32>() as u64;
    let test = build_op_test!(asm_op, &[value_a, value_b]);
    test.expect_stack(&[value_b, value_a]);
}

#[test]
fn u32assert2_fail() {
    let asm_op = "u32assert.2";
    let err = "NotU32Value";

    // vars to test
    // -------- Case 1: a > 2^32 and b > 2^32 ---------------------------------------------------
    let value_a = (1_u64 << 32) + 1;
    let value_b = value_a + 2;
    let test = build_op_test!(asm_op, &[value_a, value_b]);
    test.expect_error(TestError::ExecutionError(err));

    // -------- Case 2: a > 2^32 and b < 2^32 ---------------------------------------------------
    let value_a = (1_u64 << 32) + 1;
    let value_b = 1_u64;
    let test = build_op_test!(asm_op, &[value_a, value_b]);
    test.expect_error(TestError::ExecutionError(err));

    // --------- Case 3: a < 2^32 and b > 2^32 --------------------------------------------------
    let value_b = (1_u64 << 32) + 1;
    let value_a = 1_u64;
    let test = build_op_test!(asm_op, &[value_a, value_b]);
    test.expect_error(TestError::ExecutionError(err));
}

#[test]
fn u32assertn_fail() {
    let asm_op = "u32assert.3";

    let test = build_op_test!(asm_op, &[2, 1]);
    test.expect_error(TestError::AssemblyError("parameter"));
}

#[test]
fn u32assertw() {
    // assertion passes and leaves the stack unchanged if each element of the word < 2^32
    let asm_op = "u32assertw";

    let test = build_op_test!(asm_op, &[2, 3, 4, 5]);
    test.expect_stack(&[5, 4, 3, 2]);
}

#[test]
fn u32assertw_fail() {
    // fails if any element in the word >= 2^32
    let asm_op = "u32assertw";
    let err = "NotU32Value";

    // --- any one of the inputs inputs >= 2^32 (out of bounds) -----------------------------------
    test_inputs_out_of_bounds(asm_op, WORD_LEN);

    // --- all elements out of range --------------------------------------------------------------
    let test = build_op_test!(asm_op, &[U32_BOUND; WORD_LEN]);
    test.expect_error(TestError::ExecutionError(err));
}

#[test]
fn u32cast() {
    let asm_op = "u32cast";

    // --- a < 2^32 -------------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[1]);
    test.expect_stack(&[1]);

    // --- a > 2^32 -------------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[U32_BOUND]);
    test.expect_stack(&[0]);

    // --- rest of stack isn't affected -----------------------------------------------------------
    let a = rand_value();
    let b = rand_value();

    let test = build_op_test!(asm_op, &[a, b]);
    test.expect_stack(&[b % U32_BOUND, a]);
}

#[test]
fn u32split() {
    let asm_op = "u32split";

    // --- low bits set, no high bits set ---------------------------------------------------------
    let test = build_op_test!(asm_op, &[1]);
    test.expect_stack(&[0, 1]);

    // --- high bits set, no low bits set ---------------------------------------------------------
    let test = build_op_test!(asm_op, &[U32_BOUND]);
    test.expect_stack(&[1, 0]);

    // --- high bits and low bits set -------------------------------------------------------------
    let test = build_op_test!(asm_op, &[U32_BOUND + 1]);
    test.expect_stack(&[1, 1]);

    // --- rest of stack isn't affected -----------------------------------------------------------
    let a = rand_value();
    let b = rand_value();
    let expected_hi = b >> 32;
    let expected_lo = b % U32_BOUND;

    let test = build_op_test!(asm_op, &[a, b]);
    test.expect_stack(&[expected_hi, expected_lo, a]);
}

// U32 OPERATIONS TESTS - RANDOMIZED - CONVERSIONS AND TESTS
// ================================================================================================
proptest! {
    #[test]
    fn u32test_proptest(value in any::<u64>()) {
        let asm_op = "u32test";

        // check to see if the value of the element will be a valid u32
        let expected_result = if value % Felt::MODULUS < U32_BOUND { 1 } else { 0 };

        let test = build_op_test!(asm_op, &[value]);
        test.prop_expect_stack(&[expected_result, value])?;
    }

    #[test]
    fn u32testw_proptest(word in prop_randw::<u32>()) {
        let asm_op="u32testw";

        // should leave a 1 on the stack since all values in the word are valid u32 values
        let values: Vec<u64> = word.iter().map(|a| *a as u64).collect();
        let mut expected = values.clone();
        // push the expected result
        expected.push(1);
        // reverse the values to put the expected array in stack order
        expected.reverse();

        let test = build_op_test!(asm_op, &values);
        test.prop_expect_stack(&expected)?;
    }

    #[test]
    fn u32assert_proptest(value in any::<u32>()) {
        let asm_op = "u32assert";

        // assertion passes and leaves the stack unchanged if a < 2^32
        let test = build_op_test!(asm_op, &[value as u64]);
        test.prop_expect_stack(&[value as u64])?;
    }

    #[test]
    fn u32assertw_proptest(word in prop_randw::<u32>()) {
        let asm_op = "u32assertw";

        // should pass and leave the stack unchanged if a < 2^32 for all values in the word
        let values: Vec<u64> = word.iter().map(|a| *a as u64).collect();
        let mut expected = values.clone();
        // reverse the values to put the expected array in stack order
        expected.reverse();

        let test = build_op_test!(asm_op, &values);
        test.prop_expect_stack(&expected)?;
}

    #[test]
    fn u32cast_proptest(value in any::<u64>()) {
        let asm_op = "u32cast";

        // expected result will be mod 2^32 applied to a field element
        // so the field modulus should be applied first
        let expected_result = value % Felt::MODULUS % U32_BOUND;

        let test = build_op_test!(asm_op, &[value]);
        test.prop_expect_stack(&[expected_result])?;
    }

    #[test]
    fn u32split_proptest(value in any::<u64>()) {
        let asm_op = "u32split";

        // expected result will be mod 2^32 applied to a field element
        // so the field modulus must be applied first
        let felt_value = value % Felt::MODULUS;
        let expected_b = felt_value >> 32;
        let expected_c = felt_value as u32 as u64;

        let test = build_op_test!(asm_op, &[value, value]);
        test.prop_expect_stack(&[expected_b, expected_c, value])?;
    }
}
