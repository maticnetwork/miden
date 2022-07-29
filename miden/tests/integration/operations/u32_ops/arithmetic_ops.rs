use super::{
    build_op_test, test_param_out_of_bounds, test_unchecked_execution, TestError, U32_BOUND,
};
use proptest::prelude::*;
use rand_utils::rand_value;

// U32 OPERATIONS TESTS - MANUAL - ARITHMETIC OPERATIONS
// ================================================================================================

#[test]
fn u32checked_add() {
    let asm_op = "u32checked_add";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[1, 2]);
    test.expect_stack(&[3]);

    // --- random values --------------------------------------------------------------------------
    // test using u16 values to ensure there's no overflow so the result is valid.
    let a = rand_value::<u64>() as u16;
    let b = rand_value::<u64>() as u16;
    let expected = a as u64 + b as u64;

    let test = build_op_test!(asm_op, &[a as u64, b as u64]);
    test.expect_stack(&[expected]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let c = rand_value::<u64>();
    let test = build_op_test!(asm_op, &[c, a as u64, b as u64]);
    test.expect_stack(&[expected, c]);
}

#[test]
fn u32checked_add_fail() {
    let asm_op = "u32checked_add";

    // should fail if a >= 2^32
    let test = build_op_test!(asm_op, &[U32_BOUND, 0]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail if b >= 2^32
    let test = build_op_test!(asm_op, &[0, U32_BOUND]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail if a + b >= 2^32
    let a = u32::MAX;
    let b = 1_u64;
    let test = build_op_test!(asm_op, &[a as u64, b]);
    test.expect_error(TestError::ExecutionError("FailedAssertion"));
}

#[test]
fn u32checked_add_b() {
    let build_asm_op = |param: u16| format!("u32checked_add.{}", param);

    // --- simple cases ----------------------------------------------------------------------------
    let test = build_op_test!(build_asm_op(2).as_str(), &[1]);
    test.expect_stack(&[3]);

    let test = build_op_test!(build_asm_op(0).as_str(), &[1]);
    test.expect_stack(&[1]);

    // --- random values --------------------------------------------------------------------------
    // test using u16 values to ensure there's no overflow so the result is valid.
    let a = rand_value::<u64>() as u16;
    let b = rand_value::<u64>() as u16;
    let expected = a as u64 + b as u64;

    let test = build_op_test!(build_asm_op(b).as_str(), &[a as u64]);
    test.expect_stack(&[expected]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let c = rand_value::<u64>();

    let test = build_op_test!(build_asm_op(b).as_str(), &[c, a as u64]);
    test.expect_stack(&[expected, c]);
}

#[test]
fn u32checked_add_b_fail() {
    let build_asm_op = |param: u64| format!("u32checked_add.{}", param);

    // should fail during execution if a >= 2^32.
    let test = build_op_test!(build_asm_op(0).as_str(), &[U32_BOUND]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail during compilation if b >= 2^32.
    test_param_out_of_bounds("u32checked_add", U32_BOUND);

    // should fail if a + b >= 2^32.
    let a = u32::MAX;
    let b = 1_u64;

    let test = build_op_test!(build_asm_op(b).as_str(), &[a as u64]);
    test.expect_error(TestError::ExecutionError("FailedAssertion"));
}

#[test]
fn u32wrapping_add() {
    let asm_op = "u32wrapping_add";

    // --- (a + b) < 2^32 -------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[1, 2]);
    test.expect_stack(&[3]);

    // --- (a + b) = 2^32 -------------------------------------------------------------------------
    let a = u32::MAX;
    let b = 1_u64;
    // c should be 0, since sum is overflowed
    let test = build_op_test!(asm_op, &[a as u64, b]);
    test.expect_stack(&[0]);

    // --- (a + b) > 2^32 -------------------------------------------------------------------------
    let a = 2_u64;
    let b = u32::MAX;
    // c should be the sum mod 2^32
    let test = build_op_test!(asm_op, &[a, b as u64]);
    test.expect_stack(&[1]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let b = rand_value::<u32>();
    let c = a.wrapping_add(b);
    let test = build_op_test!(asm_op, &[a as u64, b as u64]);
    test.expect_stack(&[c as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let e = rand_value::<u64>();
    let test = build_op_test!(asm_op, &[e, a as u64, b as u64]);
    test.expect_stack(&[c as u64, e]);
}

#[test]
fn u32wrapping_add_b() {
    let build_asm_op = |param: u64| format!("u32wrapping_add.{}", param);

    // --- (a + b) < 2^32 -------------------------------------------------------------------------
    let test = build_op_test!(build_asm_op(2), &[1]);
    test.expect_stack(&[3]);

    // --- (a + b) = 2^32 -------------------------------------------------------------------------
    let a = u32::MAX as u64;
    // c should be 0, since sum is overflowed
    let test = build_op_test!(build_asm_op(1), &[a]);
    test.expect_stack(&[0]);

    // --- (a + b) > 2^32 -------------------------------------------------------------------------
    let a = 2_u64;
    let b = u32::MAX as u64;
    // c should be the sum mod 2^32
    let test = build_op_test!(build_asm_op(b), &[a]);
    test.expect_stack(&[1]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let b = rand_value::<u32>();
    let c = a.wrapping_add(b);
    let test = build_op_test!(build_asm_op(b as u64), &[a as u64]);
    test.expect_stack(&[c as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let e = rand_value::<u64>();
    let test = build_op_test!(build_asm_op(b as u64), &[e, a as u64]);
    test.expect_stack(&[c as u64, e]);
}

#[test]
fn u32overflowing_add() {
    let asm_op = "u32overflowing_add";

    // --- (a + b) < 2^32 -------------------------------------------------------------------------
    // c = a + b and d should be unset, since there was no overflow.
    let test = build_op_test!(asm_op, &[1, 2]);
    test.expect_stack(&[0, 3]);

    // --- (a + b) = 2^32 -------------------------------------------------------------------------
    let a = u32::MAX;
    let b = 1_u64;
    // c should be the sum mod 2^32 and d should be set to signal overflow.
    let test = build_op_test!(asm_op, &[a as u64, b]);
    test.expect_stack(&[1, 0]);

    // --- (a + b) > 2^32 -------------------------------------------------------------------------
    let a = 2_u64;
    let b = u32::MAX;
    // c should be the sum mod 2^32 and d should be set to signal overflow.
    let test = build_op_test!(asm_op, &[a, b as u64]);
    test.expect_stack(&[1, 1]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let b = rand_value::<u32>();
    let (c, overflow) = a.overflowing_add(b);
    let d = if overflow { 1 } else { 0 };
    let test = build_op_test!(asm_op, &[a as u64, b as u64]);
    test.expect_stack(&[d, c as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let e = rand_value::<u64>();
    let test = build_op_test!(asm_op, &[e, a as u64, b as u64]);
    test.expect_stack(&[d, c as u64, e]);

    // should not fail when inputs are out of bounds.
    test_unchecked_execution(asm_op, 2);
}

#[test]
fn u32overflowing_add3() {
    let asm_op = "u32overflowing_add3";

    // --- test correct execution -----------------------------------------------------------------
    // --- (a + b + c) < 2^32 where c = 0 ---------------------------------------------------------
    // d = a + b + c and e should be unset, since there was no overflow.
    let test = build_op_test!(asm_op, &[0, 1, 2]);
    test.expect_stack(&[0, 3]);

    // --- (a + b + c) < 2^32 where c = 1 ---------------------------------------------------------
    // d = a + b + c and e should be unset, since there was no overflow.
    let test = build_op_test!(asm_op, &[1, 2, 3]);
    test.expect_stack(&[0, 6]);

    // --- (a + b + c) = 2^32 ---------------------------------------------------------------------
    let a = u32::MAX;
    let b = 1_u64;
    // d should be the sum mod 2^32 and e should be set to signal overflow.
    let test = build_op_test!(asm_op, &[0, a as u64, b]);
    test.expect_stack(&[1, 0]);

    // --- (a + b + c) > 2^32 ---------------------------------------------------------------------
    let a = 1_u64;
    let b = u32::MAX;
    // d should be the sum mod 2^32 and e should be set to signal overflow.
    let test = build_op_test!(asm_op, &[1, a, b as u64]);
    test.expect_stack(&[1, 1]);

    // --- random u32 values with c = 0 -----------------------------------------------------------
    let a = rand_value::<u32>();
    let b = rand_value::<u32>();
    let c = 0_u64;
    let (d, overflow) = a.overflowing_add(b);
    let e = if overflow { 1 } else { 0 };
    let test = build_op_test!(asm_op, &[c, a as u64, b as u64]);
    test.expect_stack(&[e, d as u64]);

    // --- random u32 values with c = 1 -----------------------------------------------------------
    let a = rand_value::<u32>();
    let b = rand_value::<u32>();
    let c = 1_u32;
    let (d, overflow_b) = a.overflowing_add(b);
    let (d, overflow_c) = d.overflowing_add(c);
    let e = if overflow_b || overflow_c { 1 } else { 0 };
    let test = build_op_test!(asm_op, &[c as u64, a as u64, b as u64]);
    test.expect_stack(&[e, d as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let f = rand_value::<u64>();
    let test = build_op_test!(asm_op, &[f, c as u64, a as u64, b as u64]);
    test.expect_stack(&[e, d as u64, f]);

    // --- test that out of bounds inputs do not cause a failure ----------------------------------

    // should not fail if a >= 2^32.
    let test = build_op_test!(asm_op, &[0, 0, U32_BOUND]);
    assert!(test.execute().is_ok());

    // should not fail if b >= 2^32.
    let test = build_op_test!(asm_op, &[0, U32_BOUND, 0]);
    assert!(test.execute().is_ok());

    // should not fail if c >= 2^32.
    let test = build_op_test!(asm_op, &[U32_BOUND, 0, 0]);
    assert!(test.execute().is_ok());
}

#[test]
fn u32checked_sub() {
    let asm_op = "u32checked_sub";

    // --- simple cases ---------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[1, 1]);
    test.expect_stack(&[0]);

    let test = build_op_test!(asm_op, &[2, 1]);
    test.expect_stack(&[1]);

    // --- random u32 values ----------------------------------------------------------------------
    let val1 = rand_value::<u32>();
    let val2 = rand_value::<u32>();
    // assign the larger value to a and the smaller value to b.
    let (a, b) = if val1 >= val2 {
        (val1, val2)
    } else {
        (val2, val1)
    };
    let expected = a - b;

    let test = build_op_test!(asm_op, &[a as u64, b as u64]);
    test.expect_stack(&[expected as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let c = rand_value::<u64>();

    let test = build_op_test!(asm_op, &[c, a as u64, b as u64]);
    test.expect_stack(&[expected as u64, c]);
}

#[test]
fn u32checked_sub_fail() {
    let asm_op = "u32checked_sub";

    // should fail if a >= 2^32.
    let test = build_op_test!(asm_op, &[U32_BOUND, 0]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail if b >= 2^32.
    let test = build_op_test!(asm_op, &[0, U32_BOUND]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail if a < b.
    let a = 1_u64;
    let b = 2_u64;
    let test = build_op_test!(asm_op, &[a, b]);
    test.expect_error(TestError::ExecutionError("FailedAssertion"));
}

#[test]
fn u32checked_sub_b() {
    let build_asm_op = |param: u32| format!("u32checked_sub.{}", param);

    // --- simple cases ---------------------------------------------------------------------------
    let test = build_op_test!(build_asm_op(1).as_str(), &[2]);
    test.expect_stack(&[1]);

    let test = build_op_test!(build_asm_op(1).as_str(), &[1]);
    test.expect_stack(&[0]);

    // --- random u32 values ----------------------------------------------------------------------
    let val1 = rand_value::<u32>();
    let val2 = rand_value::<u32>();
    // assign the larger value to a and the smaller value to b.
    let (a, b) = if val1 >= val2 {
        (val1, val2)
    } else {
        (val2, val1)
    };
    let expected = a - b;

    let test = build_op_test!(build_asm_op(b).as_str(), &[a as u64]);
    test.expect_stack(&[expected as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let c = rand_value::<u64>();

    let test = build_op_test!(build_asm_op(b).as_str(), &[c, a as u64]);
    test.expect_stack(&[expected as u64, c]);
}

#[test]
fn u32checked_sub_b_fail() {
    let build_asm_op = |param: u64| format!("u32checked_sub.{}", param);

    // should fail during execution if a >= 2^32.
    let test = build_op_test!(build_asm_op(0).as_str(), &[U32_BOUND]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail during compilation if b >= 2^32.
    test_param_out_of_bounds("u32checked_sub", U32_BOUND);

    // should fail if a < b.
    let a = 1_u64;
    let b = 2_u64;
    let test = build_op_test!(build_asm_op(b).as_str(), &[a]);
    test.expect_error(TestError::ExecutionError("FailedAssertion"));
}

#[test]
fn u32wrapping_sub() {
    let asm_op = "u32wrapping_sub";

    // --- a > b -------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[2, 1]);
    test.expect_stack(&[1]);

    // --- a = b -------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[1, 1]);
    test.expect_stack(&[0]);

    // --- a < b -------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[1, 2]);
    test.expect_stack(&[u32::MAX as u64]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let b = rand_value::<u32>();
    let c = a.wrapping_sub(b);
    let test = build_op_test!(asm_op, &[a as u64, b as u64]);
    test.expect_stack(&[c as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let e = rand_value::<u64>();
    let test = build_op_test!(asm_op, &[e, a as u64, b as u64]);
    test.expect_stack(&[c as u64, e]);
}

#[test]
fn u32wrapping_sub_b() {
    let build_asm_op = |param: u64| format!("u32wrapping_sub.{}", param);

    // --- a > b -------------------------------------------------------------------------
    let test = build_op_test!(build_asm_op(1), &[2]);
    test.expect_stack(&[1]);

    // --- a = b -------------------------------------------------------------------------
    let test = build_op_test!(build_asm_op(1), &[1]);
    test.expect_stack(&[0]);

    // --- a < b -------------------------------------------------------------------------
    let test = build_op_test!(build_asm_op(2), &[1]);
    test.expect_stack(&[u32::MAX as u64]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let b = rand_value::<u32>();
    let c = a.wrapping_sub(b);
    let test = build_op_test!(build_asm_op(b as u64), &[a as u64]);
    test.expect_stack(&[c as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let e = rand_value::<u64>();
    let test = build_op_test!(build_asm_op(b as u64), &[e, a as u64]);
    test.expect_stack(&[c as u64, e]);
}

#[test]
fn u32overflowing_sub() {
    let asm_op = "u32overflowing_sub";

    // --- a > b -------------------------------------------------------------------------
    // c = a - b and d should be unset, since there was no arithmetic overflow.
    let test = build_op_test!(asm_op, &[2, 1]);
    test.expect_stack(&[0, 1]);

    // --- a = b -------------------------------------------------------------------------
    // c = a - b and d should be unset, since there was no arithmetic overflow.
    let test = build_op_test!(asm_op, &[1, 1]);
    test.expect_stack(&[0, 0]);

    // --- a < b -------------------------------------------------------------------------
    // c = a - b % 2^32 and d should be set, since there was arithmetic overflow.
    let test = build_op_test!(asm_op, &[1, 2]);
    test.expect_stack(&[1, u32::MAX as u64]);

    // --- random u32 values: a >= b --------------------------------------------------------------
    let val1 = rand_value::<u32>();
    let val2 = rand_value::<u32>();
    let (a, b) = if val1 >= val2 {
        (val1, val2)
    } else {
        (val2, val1)
    };
    let c = a - b;
    let test = build_op_test!(asm_op, &[a as u64, b as u64]);
    test.expect_stack(&[0, c as u64]);

    // --- random u32 values: a < b ---------------------------------------------------------------
    let val1 = rand_value::<u32>();
    let val2 = rand_value::<u32>();
    let (a, b) = if val1 >= val2 {
        (val2, val1)
    } else {
        (val1, val2)
    };
    let (c, _) = a.overflowing_sub(b);
    let d = 1;
    let test = build_op_test!(asm_op, &[a as u64, b as u64]);
    test.expect_stack(&[d, c as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let e = rand_value::<u64>();
    let test = build_op_test!(asm_op, &[e, a as u64, b as u64]);
    test.expect_stack(&[d, c as u64, e]);

    // should not fail when inputs are out of bounds.
    test_unchecked_execution(asm_op, 2);
}

#[test]
fn u32checked_mul() {
    let asm_op = "u32checked_mul";

    // --- simple cases ---------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[1, 0]);
    test.expect_stack(&[0]);

    let test = build_op_test!(asm_op, &[5, 1]);
    test.expect_stack(&[5]);

    let test = build_op_test!(asm_op, &[2, 5]);
    test.expect_stack(&[10]);

    // --- random values --------------------------------------------------------------------------
    // test using u16 values to ensure there's no overflow so the result is valid.
    let a = rand_value::<u16>();
    let b = rand_value::<u16>();

    let expected: u64 = a as u64 * b as u64;
    let test = build_op_test!(asm_op, &[a as u64, b as u64]);
    test.expect_stack(&[expected]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let c = rand_value::<u64>();
    let test = build_op_test!(asm_op, &[c, a as u64, b as u64]);
    test.expect_stack(&[expected, c]);
}

#[test]
fn u32checked_mul_fail() {
    let asm_op = "u32checked_mul";

    // should fail if a >= 2^32.
    let test = build_op_test!(asm_op, &[U32_BOUND, 0]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail if b >= 2^32.
    let test = build_op_test!(asm_op, &[0, U32_BOUND]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail if a * b  >= 2^32.
    let a = u32::MAX as u64;
    let b = 2_u64;
    let test = build_op_test!(asm_op, &[a, b]);
    test.expect_error(TestError::ExecutionError("FailedAssertion"));
}

#[test]
fn u32checked_mul_b() {
    let build_asm_op = |param: u16| format!("u32checked_mul.{}", param);

    // --- simple cases ---------------------------------------------------------------------------
    let test = build_op_test!(build_asm_op(0).as_str(), &[1]);
    test.expect_stack(&[0]);

    let test = build_op_test!(build_asm_op(1).as_str(), &[5]);
    test.expect_stack(&[5]);

    let test = build_op_test!(build_asm_op(5).as_str(), &[2]);
    test.expect_stack(&[10]);

    // --- random values --------------------------------------------------------------------------
    // test using u16 values to ensure there's no overflow so the result is valid.
    let a = rand_value::<u16>();
    let b = rand_value::<u16>();

    let expected: u64 = a as u64 * b as u64;
    let test = build_op_test!(build_asm_op(b).as_str(), &[a as u64]);
    test.expect_stack(&[expected as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let c = rand_value::<u64>();
    let test = build_op_test!(build_asm_op(5).as_str(), &[c, 10]);
    test.expect_stack(&[50, c]);
}

#[test]
fn u32checked_mul_b_fail() {
    let build_asm_op = |param: u64| format!("u32checked_mul.{}", param);

    // should fail during execution if a >= 2^32.
    let test = build_op_test!(build_asm_op(0).as_str(), &[U32_BOUND]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail during compilation if b >= 2^32.
    test_param_out_of_bounds("u32checked_mul", U32_BOUND);

    // should fail if a * b >= 2^32.
    let a = u32::MAX as u64;
    let b = u32::MAX as u64;
    let test = build_op_test!(build_asm_op(b).as_str(), &[a]);
    test.expect_error(TestError::ExecutionError("FailedAssertion"));
}

#[test]
fn u32wrapping_mul() {
    let asm_op = "u32wrapping_mul";

    // --- no overflow ----------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[1, 2]);
    test.expect_stack(&[2]);

    // --- overflow once --------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[U32_BOUND / 2, 2]);
    test.expect_stack(&[0]);

    // --- multiple overflows ---------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[U32_BOUND / 2, 4]);
    test.expect_stack(&[0]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let b = rand_value::<u32>();
    let c = a.wrapping_mul(b);
    let test = build_op_test!(asm_op, &[a as u64, b as u64]);
    test.expect_stack(&[c as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let e = rand_value::<u64>();
    let test = build_op_test!(asm_op, &[e, a as u64, b as u64]);
    test.expect_stack(&[c as u64, e]);
}

#[test]
fn u32wrapping_mul_b() {
    let build_asm_op = |param: u64| format!("u32wrapping_mul.{}", param);

    // --- no overflow ----------------------------------------------------------------------------
    // c = a * b and d should be unset, since there was no arithmetic overflow.
    let test = build_op_test!(build_asm_op(2), &[1]);
    test.expect_stack(&[2]);

    // --- overflow once --------------------------------------------------------------------------
    // c = a * b and d = 1, since it overflows once.
    let test = build_op_test!(build_asm_op(2), &[U32_BOUND / 2]);
    test.expect_stack(&[0]);

    // --- multiple overflows ---------------------------------------------------------------------
    // c = a * b and d = 2, since it overflows twice.
    let test = build_op_test!(build_asm_op(4), &[U32_BOUND / 2]);
    test.expect_stack(&[0]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let b = rand_value::<u32>();
    let c = a.wrapping_mul(b);
    let test = build_op_test!(build_asm_op(b as u64), &[a as u64]);
    test.expect_stack(&[c as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let e = rand_value::<u64>();
    let test = build_op_test!(build_asm_op(b as u64), &[e, a as u64]);
    test.expect_stack(&[c as u64, e]);
}

#[test]
fn u32overflowing_mul() {
    let asm_op = "u32overflowing_mul";

    // --- no overflow ----------------------------------------------------------------------------
    // c = a * b and d should be unset, since there was no arithmetic overflow.
    let test = build_op_test!(asm_op, &[1, 2]);
    test.expect_stack(&[0, 2]);

    // --- overflow once --------------------------------------------------------------------------
    // c = a * b and d = 1, since it overflows once.
    let test = build_op_test!(asm_op, &[U32_BOUND / 2, 2]);
    test.expect_stack(&[1, 0]);

    // --- multiple overflows ---------------------------------------------------------------------
    // c = a * b and d = 2, since it overflows twice.
    let test = build_op_test!(asm_op, &[U32_BOUND / 2, 4]);
    test.expect_stack(&[2, 0]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let b = rand_value::<u32>();
    let (c, overflow) = a.overflowing_mul(b);
    let d = if !overflow {
        0
    } else {
        (a as u64 * b as u64) / U32_BOUND
    };
    let test = build_op_test!(asm_op, &[a as u64, b as u64]);
    test.expect_stack(&[d, c as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let e = rand_value::<u64>();
    let test = build_op_test!(asm_op, &[e, a as u64, b as u64]);
    test.expect_stack(&[d, c as u64, e]);

    // should not fail when inputs are out of bounds.
    test_unchecked_execution(asm_op, 2);
}

#[test]
fn u32overflowing_madd() {
    let asm_op = "u32overflowing_madd";

    // --- no overflow ----------------------------------------------------------------------------
    // d = a * b + c and e should be unset, since there was no arithmetic overflow.
    let test = build_op_test!(asm_op, &[1, 0, 0]);
    test.expect_stack(&[0, 1]);

    let test = build_op_test!(asm_op, &[3, 1, 2]);
    test.expect_stack(&[0, 5]);

    // --- overflow once --------------------------------------------------------------------------
    // c = a * b and d = 1, since it overflows once
    let test = build_op_test!(asm_op, &[1, U32_BOUND / 2, 2]);
    test.expect_stack(&[1, 1]);

    // --- multiple overflows ---------------------------------------------------------------------
    // c = a * b and d = 2, since it overflows twice
    let test = build_op_test!(asm_op, &[1, U32_BOUND / 2, 4]);
    test.expect_stack(&[2, 1]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let b = rand_value::<u32>();
    let c = rand_value::<u32>();
    let madd = a as u64 * b as u64 + c as u64;
    let d = madd % U32_BOUND;
    let e = madd / U32_BOUND;
    let test = build_op_test!(asm_op, &[c as u64, a as u64, b as u64]);
    test.expect_stack(&[e, d]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let f = rand_value::<u64>();
    let test = build_op_test!(asm_op, &[f, c as u64, a as u64, b as u64]);
    test.expect_stack(&[e, d, f]);

    // should not fail when inputs are out of bounds.
    test_unchecked_execution(asm_op, 3);
}

#[test]
fn u32checked_div() {
    let asm_op = "u32checked_div";

    test_div(asm_op);
}

#[test]
fn u32checked_div_fail() {
    let asm_op = "u32checked_div";

    // should fail if a >= 2^32
    let test = build_op_test!(asm_op, &[U32_BOUND, 1]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail if b >= 2^32
    let test = build_op_test!(asm_op, &[1, U32_BOUND]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail if b == 0
    let test = build_op_test!(asm_op, &[1, 0]);
    test.expect_error(TestError::ExecutionError("DivideByZero"));
}

#[test]
fn u32checked_div_b() {
    let build_asm_op = |param: u32| format!("u32checked_div.{}", param);

    // --- simple cases ---------------------------------------------------------------------------
    let test = build_op_test!(build_asm_op(1).as_str(), &[0]);
    test.expect_stack(&[0]);

    // division with no remainder
    let test = build_op_test!(build_asm_op(1).as_str(), &[2]);
    test.expect_stack(&[2]);

    // division with remainder
    let test = build_op_test!(build_asm_op(2).as_str(), &[1]);
    test.expect_stack(&[0]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let mut b = rand_value::<u32>();
    if b == 0 {
        // ensure we're not using a failure case.
        b += 1;
    }
    let expected = (a / b) as u64;

    let test = build_op_test!(build_asm_op(b).as_str(), &[a as u64]);
    test.expect_stack(&[expected]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let c = rand_value::<u64>();
    let test = build_op_test!(build_asm_op(b).as_str(), &[c, a as u64]);
    test.expect_stack(&[expected as u64, c]);
}

#[test]
fn u32checked_div_b_fail() {
    let build_asm_op = |param: u64| format!("u32checked_div.{}", param);

    // should fail during execution if a >= 2^32.
    let test = build_op_test!(build_asm_op(1).as_str(), &[U32_BOUND]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail during compilation if b >= 2^32.
    test_param_out_of_bounds("u32checked_div", U32_BOUND);

    // should fail during compilation if b = 0.
    let test = build_op_test!(build_asm_op(0).as_str());
    test.expect_error(TestError::AssemblyError("parameter"));
}

#[test]
fn u32unchecked_div() {
    let asm_op = "u32unchecked_div";

    // should push d = (a * b) / 2^32 onto the stack.
    test_div(asm_op);

    // should not fail when inputs are out of bounds.
    test_unchecked_execution(asm_op, 2);
}

#[test]
fn u32unchecked_div_fail() {
    let asm_op = "u32unchecked_div";

    // should fail if b == 0.
    let test = build_op_test!(asm_op, &[1, 0]);
    test.expect_error(TestError::ExecutionError("DivideByZero"));
}

#[test]
fn u32checked_mod() {
    let asm_op = "u32checked_mod";

    test_mod(asm_op);
}

#[test]
fn u32checked_mod_fail() {
    let asm_op = "u32checked_mod";

    // should fail if a >= 2^32
    let test = build_op_test!(asm_op, &[U32_BOUND, 1]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail if b >= 2^32
    let test = build_op_test!(asm_op, &[1, U32_BOUND]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail if b == 0
    let test = build_op_test!(asm_op, &[1, 0]);
    test.expect_error(TestError::ExecutionError("DivideByZero"));
}

#[test]
fn u32checked_mod_b() {
    let build_asm_op = |param: u32| format!("u32checked_mod.{}", param);

    // --- simple cases ---------------------------------------------------------------------------
    let test = build_op_test!(build_asm_op(5).as_str(), &[10]);
    test.expect_stack(&[0]);

    let test = build_op_test!(build_asm_op(5).as_str(), &[11]);
    test.expect_stack(&[1]);

    let test = build_op_test!(build_asm_op(11).as_str(), &[5]);
    test.expect_stack(&[5]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let mut b = rand_value::<u32>();
    if b == 0 {
        // ensure we're not using a failure case.
        b += 1;
    }
    let expected = a % b;

    let test = build_op_test!(build_asm_op(b).as_str(), &[a as u64]);
    test.expect_stack(&[expected as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let c = rand_value::<u64>();

    let test = build_op_test!(build_asm_op(b).as_str(), &[c, a as u64]);
    test.expect_stack(&[expected as u64, c]);
}

#[test]
fn u32checked_mod_b_fail() {
    let build_asm_op = |param: u64| format!("u32checked_mod.{}", param);

    // should fail during execution if a >= 2^32.
    let test = build_op_test!(build_asm_op(1).as_str(), &[U32_BOUND]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail during compilation if b >= 2^32.
    test_param_out_of_bounds("u32checked_mod", U32_BOUND);

    // should fail during compilation if b = 0.
    let test = build_op_test!(build_asm_op(0).as_str());
    test.expect_error(TestError::AssemblyError("parameter"));
}

#[test]
fn u32unchecked_mod() {
    let asm_op = "u32unchecked_mod";

    test_mod(asm_op);

    // should not fail when inputs are out of bounds.
    test_unchecked_execution(asm_op, 2);
}

#[test]
fn u32unchecked_mod_fail() {
    let asm_op = "u32unchecked_mod";

    // should fail if b == 0
    let test = build_op_test!(asm_op, &[1, 0]);
    test.expect_error(TestError::ExecutionError("DivideByZero"));
}

#[test]
fn u32checked_divmod() {
    let asm_op = "u32checked_divmod";

    test_divmod(asm_op);
}

#[test]
fn u32checked_divmod_fail() {
    let asm_op = "u32checked_divmod";

    // should fail if a >= 2^32
    let test = build_op_test!(asm_op, &[U32_BOUND, 1]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail if b >= 2^32
    let test = build_op_test!(asm_op, &[1, U32_BOUND]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail if b == 0
    let test = build_op_test!(asm_op, &[1, 0]);
    test.expect_error(TestError::ExecutionError("DivideByZero"));
}

#[test]
fn u32checked_divmod_b() {
    let build_asm_op = |param: u32| format!("u32checked_divmod.{}", param);

    // --- simple cases ---------------------------------------------------------------------------
    let test = build_op_test!(build_asm_op(1).as_str(), &[0]);
    test.expect_stack(&[0, 0]);

    // division with no remainder
    let test = build_op_test!(build_asm_op(1).as_str(), &[2]);
    test.expect_stack(&[0, 2]);

    // division with remainder
    let test = build_op_test!(build_asm_op(2).as_str(), &[1]);
    test.expect_stack(&[1, 0]);
    let test = build_op_test!(build_asm_op(2).as_str(), &[3]);
    test.expect_stack(&[1, 1]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let mut b = rand_value::<u32>();
    if b == 0 {
        // ensure we're not using a failure case.
        b += 1;
    }
    let quot = (a / b) as u64;
    let rem = (a % b) as u64;
    let test = build_op_test!(build_asm_op(b).as_str(), &[a as u64]);
    test.expect_stack(&[rem, quot]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let e = rand_value::<u64>();
    let test = build_op_test!(build_asm_op(b).as_str(), &[e, a as u64]);
    test.expect_stack(&[rem, quot, e]);
}

#[test]
fn u32checked_divmod_b_fail() {
    let build_asm_op = |param: u64| format!("u32checked_divmod.{}", param);

    // should fail during execution if a >= 2^32.
    let test = build_op_test!(build_asm_op(1).as_str(), &[U32_BOUND]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // should fail during compilation if b >= 2^32.
    test_param_out_of_bounds("u32checked_divmod", U32_BOUND);

    // should fail during compilation if b = 0.
    let test = build_op_test!(build_asm_op(0).as_str());
    test.expect_error(TestError::AssemblyError("parameter"));
}

#[test]
fn u32unchecked_divmod() {
    let asm_op = "u32unchecked_divmod";

    test_divmod(asm_op);

    // should not fail when inputs are out of bounds.
    test_unchecked_execution(asm_op, 2);
}

#[test]
fn u32unchecked_divmod_fail() {
    let asm_op = "u32unchecked_divmod";

    // should fail if b == 0.
    let test = build_op_test!(asm_op, &[1, 0]);
    test.expect_error(TestError::ExecutionError("DivideByZero"));
}

// U32 OPERATIONS TESTS - RANDOMIZED - ARITHMETIC OPERATIONS
// ================================================================================================
proptest! {
    #[test]
    fn u32checked_add_proptest(a in any::<u16>(), b in any::<u16>()) {
        let asm_op = "u32checked_add";

        let expected = a as u64 + b as u64;

        // b provided via the stack.
        let test = build_op_test!(asm_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[expected])?;

        // b provided as a parameter.
        let asm_op = format!("{}.{}", asm_op, b);
        let test = build_op_test!(&asm_op, &[a as u64]);
        test.prop_expect_stack(&[expected])?;
    }

    #[test]
    fn u32unchecked_add_proptest(a in any::<u32>(), b in any::<u32>()) {
        let wrapping_asm_op = "u32wrapping_add";
        let overflowing_asm_op = "u32overflowing_add";

        let (c, overflow) = a.overflowing_add(b);
        let d = if overflow { 1 } else { 0 };

        let test = build_op_test!(wrapping_asm_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[c as u64])?;

        let test = build_op_test!(overflowing_asm_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[d, c as u64])?;
    }

    #[test]
    fn u32overflowing_add3_proptest(a in any::<u32>(), b in any::<u32>(), c in any::<u32>()) {
        let asm_op = "u32overflowing_add3";

        let sum: u64 = u64::from(a) + u64::from(b) + u64::from(c);
        let lo = (sum as u32) as u64;
        let hi = sum >> 32;

        let test = build_op_test!(asm_op, &[c as u64, a as u64, b as u64]);
        test.prop_expect_stack(&[hi, lo])?;
    }

    #[test]
    fn u32checked_sub_proptest(val1 in any::<u32>(), val2 in any::<u32>()) {
        let asm_op = "u32checked_sub";

        // assign the larger value to a and the smaller value to b so all parameters are valid.
        let (a, b) = if val1 >= val2 {
            (val1, val2)
        } else {
            (val2, val1)
        };

        let expected = a - b;
        // b provided via the stack.
        let test = build_op_test!(asm_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[expected as u64])?;

        // b provided as a parameter.
        let asm_op = format!("{}.{}", asm_op, b);
        let test = build_op_test!(&asm_op, &[a as u64]);
        test.prop_expect_stack(&[expected as u64])?;

    }

    #[test]
    fn u32unchecked_sub_proptest(a in any::<u32>(), b in any::<u32>()) {
        let wrapping_asm_op = "u32wrapping_sub";
        let overflowing_asm_op = "u32overflowing_sub";

        // assign the larger value to a and the smaller value to b so all parameters are valid.
        let (c, overflow) = a.overflowing_sub(b);
        let d = if overflow { 1 } else { 0 };

        let test = build_op_test!(wrapping_asm_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[c as u64])?;

        let test = build_op_test!(overflowing_asm_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[d, c as u64])?;
    }

    #[test]
    fn u32checked_mul_proptest(a in any::<u16>(), b in any::<u16>()) {
        let asm_op = "u32checked_mul";

        let expected = a as u64 * b as u64;

        // b provided via the stack
        let test = build_op_test!(asm_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[expected])?;

        // b provided as a parameter
        let asm_op = format!("{}.{}", asm_op, b);
        let test = build_op_test!(&asm_op, &[a as u64]);
        test.prop_expect_stack(&[expected])?;
    }

    #[test]
    fn u32unchecked_mul_proptest(a in any::<u32>(), b in any::<u32>()) {
        let wrapping_asm_op = "u32wrapping_mul";
        let overflowing_asm_op = "u32overflowing_mul";

        let (c, overflow) = a.overflowing_mul(b);
        let d = if !overflow {
            0
        } else {
            (a as u64 * b as u64) / U32_BOUND
        };

        let test = build_op_test!(wrapping_asm_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[c as u64])?;

        let test = build_op_test!(overflowing_asm_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[d, c as u64])?;
    }

    #[test]
    fn u32overflowing_madd_proptest(a in any::<u32>(), b in any::<u32>(), c in any::<u32>()) {
        let asm_op = "u32overflowing_madd";

        let madd = a as u64 * b as u64 + c as u64;
        let d = madd % U32_BOUND;
        let e = madd / U32_BOUND;

        let test = build_op_test!(asm_op, &[c as u64, a as u64, b as u64]);
        test.prop_expect_stack(&[e, d as u64])?;
    }

    #[test]
    fn u32div_proptest(a in any::<u32>(), b in 1..u32::MAX) {
        let asm_op = "u32checked_div";

        let expected = (a / b) as u64;

        // b provided via the stack.
        let test = build_op_test!(asm_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[expected])?;

        // b provided as a parameter.
        let asm_op = format!("{}.{}", asm_op, b);
        let test = build_op_test!(&asm_op, &[a as u64]);
        test.prop_expect_stack(&[expected])?;

        // unchecked version should produce the same result for valid values.
        let asm_op = "u32unchecked_div";
        let test = build_op_test!(&asm_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[expected as u64])?;

        let asm_op = format!("{}.{}", asm_op, b);
        let test = build_op_test!(&asm_op, &[a as u64]);
        test.prop_expect_stack(&[expected as u64])?;
    }

    #[test]
    fn u32mod_proptest(a in any::<u32>(), b in 1..u32::MAX) {
        let base_op = "u32checked_mod";

        let expected = a % b;

        // b provided via the stack
        let test = build_op_test!(base_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[expected as u64])?;

        // b provided as a parameter
        let asm_op = format!("{}.{}", base_op, b);
        let test = build_op_test!(&asm_op, &[a as u64]);
        test.prop_expect_stack(&[expected as u64])?;

        // unchecked version should produce the same result for valid values.
        let asm_op = "u32unchecked_mod";
        let test = build_op_test!(&asm_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[expected as u64])?;

        let asm_op = format!("{}.{}", asm_op, b);
        let test = build_op_test!(&asm_op, &[a as u64]);
        test.prop_expect_stack(&[expected as u64])?;
    }

    #[test]
    fn u32divmod_proptest(a in any::<u32>(), b in 1..u32::MAX) {
        let asm_op = "u32checked_divmod";

        let quot = (a / b) as u64;
        let rem = (a % b) as u64;

        // b provided via the stack.
        let test = build_op_test!(asm_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[rem, quot])?;

        // b provided as a parameter.
        let asm_op = format!("{}.{}", asm_op, b);
        let test = build_op_test!(&asm_op, &[a as u64]);
        test.prop_expect_stack(&[rem, quot])?;

        // unchecked version should produce the same result for valid values.
        let asm_op = "u32unchecked_divmod";
        let test = build_op_test!(&asm_op, &[a as u64, b as u64]);
        test.prop_expect_stack(&[rem, quot])?;

        let asm_op = format!("{}.{}", asm_op, b);
        let test = build_op_test!(&asm_op, &[a as u64]);
        test.prop_expect_stack(&[rem, quot])?;
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// This helper function tests division without remainder for two u32 inputs for a number of simple
/// cases as well as for random values. It checks that the floor of a / b is pushed to the
/// stack. Finally, it ensures that the rest of the stack was unaffected.
fn test_div(asm_op: &str) {
    // --- simple cases ---------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[0, 1]);
    test.expect_stack(&[0]);

    let test = build_op_test!(asm_op, &[2, 1]);
    test.expect_stack(&[2]);

    let test = build_op_test!(asm_op, &[1, 2]);
    test.expect_stack(&[0]);

    let test = build_op_test!(asm_op, &[3, 2]);
    test.expect_stack(&[1]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let mut b = rand_value::<u32>();
    if b == 0 {
        // ensure we're not using a failure case.
        b += 1;
    }
    let quot = (a / b) as u64;
    let test = build_op_test!(asm_op, &[a as u64, b as u64]);
    test.expect_stack(&[quot]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let e = rand_value::<u64>();
    let test = build_op_test!(asm_op, &[e, a as u64, b as u64]);
    test.expect_stack(&[quot, e]);
}

/// This helper function tests the modulus operation for two u32 inputs for a number of simple
/// cases as well as for random values. It checks that a % b is pushed to the stack. Finally, it
/// ensures that the rest of the stack was unaffected.
fn test_mod(asm_op: &str) {
    // --- simple cases ---------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[10, 5]);
    test.expect_stack(&[0]);

    let test = build_op_test!(asm_op, &[11, 5]);
    test.expect_stack(&[1]);

    let test = build_op_test!(asm_op, &[5, 11]);
    test.expect_stack(&[5]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let mut b = rand_value::<u32>();
    if b == 0 {
        // ensure we're not using a failure case.
        b += 1;
    }
    let expected = a % b;
    let test = build_op_test!(asm_op, &[a as u64, b as u64]);
    test.expect_stack(&[expected as u64]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let c = rand_value::<u64>();
    let test = build_op_test!(asm_op, &[c, a as u64, b as u64]);
    test.expect_stack(&[expected as u64, c]);
}

/// This helper function tests division with remainder for two u32 inputs for a number of simple
/// cases as well as for random values. It checks that the floor of a / b is pushed to the
/// stack, along with the remainder a % b. Finally, it ensures that the rest of the stack was
/// unaffected.
fn test_divmod(asm_op: &str) {
    // --- simple cases ---------------------------------------------------------------------------
    let test = build_op_test!(asm_op, &[0, 1]);
    test.expect_stack(&[0, 0]);

    // division with no remainder
    let test = build_op_test!(asm_op, &[2, 1]);
    test.expect_stack(&[0, 2]);

    // division with remainder
    let test = build_op_test!(asm_op, &[1, 2]);
    test.expect_stack(&[1, 0]);
    let test = build_op_test!(asm_op, &[3, 2]);
    test.expect_stack(&[1, 1]);

    // --- random u32 values ----------------------------------------------------------------------
    let a = rand_value::<u32>();
    let mut b = rand_value::<u32>();
    if b == 0 {
        // ensure we're not using a failure case.
        b += 1;
    }
    let quot = (a / b) as u64;
    let rem = (a % b) as u64;
    let test = build_op_test!(asm_op, &[a as u64, b as u64]);
    test.expect_stack(&[rem, quot]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let e = rand_value::<u64>();
    let test = build_op_test!(asm_op, &[e, a as u64, b as u64]);
    test.expect_stack(&[rem, quot, e]);
}
