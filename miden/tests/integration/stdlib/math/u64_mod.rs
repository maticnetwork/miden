use super::{build_test, TestError};
use proptest::prelude::*;
use rand_utils::rand_value;
use std::cmp;

// ADDITION
// ------------------------------------------------------------------------------------------------

#[test]
fn wrapping_add() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a.wrapping_add(b);

    let source = "
        use.std::math::u64
        begin
            exec.u64::wrapping_add
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn checked_add() {
    let source = "
    use.std::math::u64
    begin
        exec.u64::checked_add
    end";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_test!(source, &[1, 2, 3, 4]);
    test.expect_stack(&[6, 4]);

    // --- random values --------------------------------------------------------------------------
    // test using u16 values to ensure there's no overflow so the result is valid
    let a0 = rand_value::<u64>() as u16 as u64;
    let b0 = rand_value::<u64>() as u16 as u64;
    let a1 = rand_value::<u64>() as u16 as u64;
    let b1 = rand_value::<u64>() as u16 as u64;
    let c0 = a0 + b0;
    let c1 = a1 + b1;

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn checked_add_fail() {
    let source = "
    use.std::math::u64
    begin
        exec.u64::checked_add
    end";

    // result overflow
    let a0 = rand_value::<u64>() as u32 as u64;
    let b0 = rand_value::<u64>() as u32 as u64;
    let a1 = u32::MAX as u64;
    let b1 = u32::MAX as u64;

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_error(TestError::ExecutionError("FailedAssertion"));

    //u32 limb assertion failure
    let a0 = rand_value::<u64>();
    let b0 = rand_value::<u64>();
    let a1 = u32::MAX as u64 + 1;
    let b1 = u32::MAX as u64 + 1;

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));
}

#[test]
fn overflowing_add() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::overflowing_add
        end";

    let a = rand_value::<u64>() as u32 as u64;
    let b = rand_value::<u64>() as u32 as u64;
    let (c, _) = a.overflowing_add(b);

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[0, c1, c0]);

    let a = u64::MAX;
    let b = rand_value::<u64>();
    let (c, _) = a.overflowing_add(b);

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[1, c1, c0]);
}

// SUBTRACTION
// ------------------------------------------------------------------------------------------------

#[test]
fn wrapping_sub() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a.wrapping_sub(b);

    let source = "
        use.std::math::u64
        begin
            exec.u64::wrapping_sub
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn checked_sub() {
    let source = "
    use.std::math::u64
    begin
        exec.u64::checked_sub
    end";

    // --- simple case ----------------------------------------------------------------------------
    let test = build_test!(source, &[3, 4, 1, 2]);
    test.expect_stack(&[2, 2]);

    // --- random values --------------------------------------------------------------------------
    let common = rand_value::<u64>();
    let dif = rand_value::<u64>() as u16 as u64;

    let a = common + dif;
    let b = common;
    let c = a - b;

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn checked_sub_fail() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::checked_sub
        end";

    // result underflow
    let a0 = rand_value::<u64>() as u32 as u64;
    let b0 = rand_value::<u64>() as u32 as u64;
    let a1 = u16::MAX as u64;
    let b1 = u32::MAX as u64;

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_error(TestError::ExecutionError("FailedAssertion"));

    //u32 limb assertion failure
    let a0 = rand_value::<u64>() as u64;
    let b0 = rand_value::<u64>() as u64;
    let a1 = u32::MAX as u64 + 1;
    let b1 = u32::MAX as u64 + 1;

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));
}

#[test]
fn overflowing_sub() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let (c, flag) = a.overflowing_sub(b);

    let source = "
        use.std::math::u64
        begin
            exec.u64::overflowing_sub
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[flag as u64, c1, c0]);

    let base = rand_value::<u64>() as u32 as u64;
    let diff = rand_value::<u64>() as u32 as u64;

    let a = base;
    let b = base + diff;
    let (c, _) = a.overflowing_sub(b);

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[1, c1, c0]);

    let base = rand_value::<u64>() as u32 as u64;
    let diff = rand_value::<u64>() as u32 as u64;

    let a = base + diff;
    let b = base;
    let (c, _) = a.overflowing_sub(b);

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[0, c1, c0]);
}

// MULTIPLICATION
// ------------------------------------------------------------------------------------------------

#[test]
fn wrapping_mul() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a.wrapping_mul(b);

    let source = "
        use.std::math::u64
        begin
            exec.u64::wrapping_mul
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn checked_mul() {
    let source = "
    use.std::math::u64
    begin
        exec.u64::checked_mul
    end";

    // --- simple cases ---------------------------------------------------------------------------
    let test = build_test!(source, &[1, 2, 0, 0]);
    test.expect_stack(&[0, 0]);

    let test = build_test!(source, &[0, 0, 1, 2]);
    test.expect_stack(&[0, 0]);

    let test = build_test!(source, &[5, 1, 1, 0]);
    test.expect_stack(&[1, 5]);

    let test = build_test!(source, &[5, 2, 2, 0]);
    test.expect_stack(&[4, 10]);

    // --- random values --------------------------------------------------------------------------
    let a0 = rand_value::<u64>() as u16 as u64;
    let a1 = rand_value::<u64>() as u16 as u64;
    let b0 = rand_value::<u64>() as u16 as u64;
    let b1 = 0u64;
    let c0 = a0 * b0;
    let c1 = a1 * b0;

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn checked_mul_fail() {
    let source = "
    use.std::math::u64
    begin
        exec.u64::checked_mul
    end";

    //u32 limb assertion failure
    let a0 = u32::MAX as u64 + 1;
    let b0 = u32::MAX as u64 + 1;
    let a1 = u32::MAX as u64 + 1;
    let b1 = u32::MAX as u64 + 1;

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));

    // Higher bits assertion failure (a_hi * b_hi != 0)

    let a0 = rand_value::<u64>() as u16 as u64;
    let a1 = 2u64;
    let b0 = rand_value::<u64>() as u16 as u64;
    let b1 = 3u64;

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_error(TestError::ExecutionError("FailedAssertion"));

    // result overflow
    let a0 = rand_value::<u64>() as u32 as u64;
    let a1 = u16::MAX as u64 + rand_value::<u64>() as u16 as u64;
    let b0 = u16::MAX as u64 + rand_value::<u64>() as u16 as u64;
    let b1 = 0u64;

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_error(TestError::ExecutionError("FailedAssertion"));
}

#[test]
fn overflowing_mul() {
    let source = "
    use.std::math::u64
    begin
        exec.u64::overflowing_mul
    end";

    let a = u64::MAX as u128;
    let b = u64::MAX as u128;
    let c = a.wrapping_mul(b);

    let a = u64::MAX;
    let b = u64::MAX;

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c3, c2, c1, c0) = split_u128(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c3, c2, c1, c0]);

    let a = rand_value::<u64>() as u128;
    let b = rand_value::<u64>() as u128;
    let c = a.wrapping_mul(b);

    let a = a as u64;
    let b = b as u64;

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c3, c2, c1, c0) = split_u128(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c3, c2, c1, c0]);
}

// COMPARISONS
// ------------------------------------------------------------------------------------------------

#[test]
fn unchecked_lt() {
    // test a few manual cases; randomized tests are done using proptest
    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_lt
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[0]);

    // a = 0, b = 1
    build_test!(source, &[0, 0, 1, 0]).expect_stack(&[1]);

    // a = 1, b = 0
    build_test!(source, &[1, 0, 0, 0]).expect_stack(&[0]);
}

#[test]
fn unchecked_lte() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_lte
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[1]);

    // a = 0, b = 1
    build_test!(source, &[0, 0, 1, 0]).expect_stack(&[1]);

    // a = 1, b = 0
    build_test!(source, &[1, 0, 0, 0]).expect_stack(&[0]);

    // randomized test
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = (a <= b) as u64;

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    build_test!(source, &[a0, a1, b0, b1]).expect_stack(&[c]);
}

#[test]
fn unchecked_gt() {
    // test a few manual cases; randomized tests are done using proptest
    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_gt
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[0]);

    // a = 0, b = 1
    build_test!(source, &[0, 0, 1, 0]).expect_stack(&[0]);

    // a = 1, b = 0
    build_test!(source, &[1, 0, 0, 0]).expect_stack(&[1]);
}

#[test]
fn unchecked_gte() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_gte
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[1]);

    // a = 0, b = 1
    build_test!(source, &[0, 0, 1, 0]).expect_stack(&[0]);

    // a = 1, b = 0
    build_test!(source, &[1, 0, 0, 0]).expect_stack(&[1]);

    // randomized test
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = (a >= b) as u64;

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    build_test!(source, &[a0, a1, b0, b1]).expect_stack(&[c]);
}

#[test]
fn unchecked_min() {
    // test a few manual cases; randomized tests are done using proptest
    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_min
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[0, 0]);

    // a = 1, b = 2
    build_test!(source, &[1, 0, 2, 0]).expect_stack(&[0, 1]);

    // a = 3, b = 2
    build_test!(source, &[3, 0, 2, 0]).expect_stack(&[0, 2]);
}

#[test]
fn unchecked_max() {
    // test a few manual cases; randomized tests are done using proptest
    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_max
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[0, 0]);

    // a = 1, b = 2
    build_test!(source, &[1, 0, 2, 0]).expect_stack(&[0, 2]);

    // a = 3, b = 2
    build_test!(source, &[3, 0, 2, 0]).expect_stack(&[0, 3]);
}

#[test]
fn unchecked_eq() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_eq
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[1]);

    // a = 0, b = 1
    build_test!(source, &[0, 0, 1, 0]).expect_stack(&[0]);

    // a = 1, b = 0
    build_test!(source, &[1, 0, 0, 0]).expect_stack(&[0]);

    // randomized test
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = (a == b) as u64;

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    build_test!(source, &[a0, a1, b0, b1]).expect_stack(&[c]);
}

#[test]
fn unchecked_neq() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_neq
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[0]);

    // a = 0, b = 1
    build_test!(source, &[0, 0, 1, 0]).expect_stack(&[1]);

    // a = 1, b = 0
    build_test!(source, &[1, 0, 0, 0]).expect_stack(&[1]);

    // randomized test
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = (a != b) as u64;

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    build_test!(source, &[a0, a1, b0, b1]).expect_stack(&[c]);
}

#[test]
fn unchecked_eqz() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_eqz
        end";

    // a = 0
    build_test!(source, &[0, 0]).expect_stack(&[1]);

    // a = 1
    build_test!(source, &[1, 0]).expect_stack(&[0]);

    // randomized test
    let a: u64 = rand_value();
    let c = (a == 0) as u64;

    let (a1, a0) = split_u64(a);
    build_test!(source, &[a0, a1]).expect_stack(&[c]);
}

// DIVISION
// ------------------------------------------------------------------------------------------------

#[test]
fn unchecked_div() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a / b;

    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_div
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);

    let d = a / b0;
    let (d1, d0) = split_u64(d);

    let test = build_test!(source, &[a0, a1, b0, 0]);
    test.expect_stack(&[d1, d0]);
}

// MODULO OPERATION
// ------------------------------------------------------------------------------------------------

#[test]
fn unchecked_mod() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a % b;

    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_mod
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);

    let d = a % b0;
    let (d1, d0) = split_u64(d);

    let test = build_test!(source, &[a0, a1, b0, 0]);
    test.expect_stack(&[d1, d0]);
}

// DIVMOD OPERATION
// ------------------------------------------------------------------------------------------------

#[test]
fn unchecked_divmod() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let q = a / b;
    let r = a % b;

    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_divmod
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (q1, q0) = split_u64(q);
    let (r1, r0) = split_u64(r);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[r1, r0, q1, q0]);
}

// BITWISE OPERATIONS
// ------------------------------------------------------------------------------------------------

#[test]
fn checked_and() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a & b;

    let source = "
        use.std::math::u64
        begin
            exec.u64::checked_and
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn checked_and_fail() {
    let a0: u64 = rand_value();
    let b0: u64 = rand_value();

    let a1: u64 = u32::MAX as u64 + 1;
    let b1: u64 = u32::MAX as u64 + 1;

    let source = "
        use.std::math::u64
        begin
            exec.u64::checked_and
        end";

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));
}

#[test]
fn checked_or() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a | b;

    let source = "
        use.std::math::u64
        begin
            exec.u64::checked_or
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn checked_or_fail() {
    let a0: u64 = rand_value();
    let b0: u64 = rand_value();

    let a1: u64 = u32::MAX as u64 + 1;
    let b1: u64 = u32::MAX as u64 + 1;

    let source = "
        use.std::math::u64
        begin
            exec.u64::checked_or
        end";

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));
}

#[test]
fn checked_xor() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a ^ b;

    let source = "
        use.std::math::u64
        begin
            exec.u64::checked_xor
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn checked_xor_fail() {
    let a0: u64 = rand_value();
    let b0: u64 = rand_value();

    let a1: u64 = u32::MAX as u64 + 1;
    let b1: u64 = u32::MAX as u64 + 1;

    let source = "
        use.std::math::u64
        begin
            exec.u64::checked_xor
        end";

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_error(TestError::ExecutionError("NotU32Value"));
}

#[test]
fn unchecked_shl() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_shl
        end";

    // shift by 0
    let a: u64 = rand_value();
    let (a1, a0) = split_u64(a);
    let b: u32 = 0;

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[a1, a0, 5]);

    // shift by 31 (max lower limb of b)
    let b: u32 = 31;
    let c = a.wrapping_shl(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 32 (min for upper limb of b)
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 32;
    let c = a.wrapping_shl(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 33
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 33;
    let c = a.wrapping_shl(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift 64 by 58
    let a = 64_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 58;
    let c = a.wrapping_shl(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);
}

#[test]
fn unchecked_shr() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_shr
        end";

    // shift by 0
    let a: u64 = rand_value();
    let (a1, a0) = split_u64(a);
    let b: u32 = 0;

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[a1, a0, 5]);

    // simple right shift
    build_test!(source, &[5, 1, 1, 1]).expect_stack(&[0, 2_u64.pow(31), 5]);

    // simple right shift
    build_test!(source, &[5, 3, 3, 1]).expect_stack(&[1, 2_u64.pow(31) + 1, 5]);

    // shift by 31 (max lower limb of b)
    let b: u32 = 31;
    let c = a.wrapping_shr(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 32 (min for upper limb of b)
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 32;
    let c = a.wrapping_shr(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 33
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 33;
    let c = a.wrapping_shr(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift 4294967296 by 2
    let a = 4294967296;
    let (a1, a0) = split_u64(a);
    let b: u32 = 2;
    let c = a.wrapping_shr(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);
}

#[test]
fn overflowing_shl() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::overflowing_shl
        end";

    // shl u64 to u128 to avoid overflowing
    let shl_to_u128 = |a: u64, b: u32| -> u128 { (a as u128) << b };

    // shift by 0
    let a: u64 = rand_value();
    let (a1, a0) = split_u64(a);
    let b: u32 = 0;

    let c = shl_to_u128(a, b);
    let (d1, d0, c1, c0) = split_u128(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[d1, d0, c1, c0, 5]);

    // shift by 31 (max lower limb of b)
    let b: u32 = 31;
    let c = shl_to_u128(a, b);
    let (d1, d0, c1, c0) = split_u128(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[d1, d0, c1, c0, 5]);

    // shift by 32 (min for upper limb of b)
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 32;
    let c = shl_to_u128(a, b);
    let (d1, d0, c1, c0) = split_u128(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[d1, d0, c1, c0, 5]);

    // shift by 33
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 33;
    let c = shl_to_u128(a, b);
    let (d1, d0, c1, c0) = split_u128(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[d1, d0, c1, c0, 5]);

    // shift 64 by 58
    let a = 64_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 58;
    let c = shl_to_u128(a, b);
    let (d1, d0, c1, c0) = split_u128(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[d1, d0, c1, c0, 5]);
}

#[test]
fn overflowing_shr() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::overflowing_shr
        end";

    // get bits shifted out and return 0 if b is 0 or 64
    let bits_shifted_out = |a: u64, b: u32| -> u64 {
        if b % 64 == 0 {
            0_u64
        } else {
            a.wrapping_shl(64 - b)
        }
    };

    // shift by 0
    let a: u64 = rand_value();
    let (a1, a0) = split_u64(a);
    let b: u32 = 0;

    let c = a.wrapping_shr(b);
    let (c1, c0) = split_u64(c);
    let d = bits_shifted_out(a, b);
    let (d1, d0) = split_u64(d);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[d1, d0, c1, c0, 5]);

    // shift by 31 (max lower limb of b)
    let b: u32 = 31;

    let c = a.wrapping_shr(b);
    let (c1, c0) = split_u64(c);
    let d = bits_shifted_out(a, b);
    let (d1, d0) = split_u64(d);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[d1, d0, c1, c0, 5]);

    // shift by 32 (min for upper limb of b)
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 32;
    let c = a.wrapping_shr(b);
    let (c1, c0) = split_u64(c);
    let d = bits_shifted_out(a, b);
    let (d1, d0) = split_u64(d);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[d1, d0, c1, c0, 5]);

    // shift by 33
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 33;
    let c = a.wrapping_shr(b);
    let (c1, c0) = split_u64(c);
    let d = bits_shifted_out(a, b);
    let (d1, d0) = split_u64(d);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[d1, d0, c1, c0, 5]);

    // shift 64 by 58
    let a = 64_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 58;
    let c = a.wrapping_shr(b);
    let (c1, c0) = split_u64(c);
    let d = bits_shifted_out(a, b);
    let (d1, d0) = split_u64(d);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[d1, d0, c1, c0, 5]);
}

#[test]
fn unchecked_rotl() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_rotl
        end";

    // shift by 0
    let a: u64 = rand_value();
    let (a1, a0) = split_u64(a);
    let b: u32 = 0;

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[a1, a0, 5]);

    // shift by 31 (max lower limb of b)
    let b: u32 = 31;
    let c = a.rotate_left(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 32 (min for upper limb of b)
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 32;
    let c = a.rotate_left(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 33
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 33;
    let c = a.rotate_left(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift 64 by 58
    let a = 64_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 58;
    let c = a.rotate_left(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);
}

#[test]
fn unchecked_rotr() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_rotr
        end";

    // shift by 0
    let a: u64 = rand_value();
    let (a1, a0) = split_u64(a);
    let b: u32 = 0;

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[a1, a0, 5]);

    // shift by 31 (max lower limb of b)
    let b: u32 = 31;
    let c = a.rotate_right(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 32 (min for upper limb of b)
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 32;
    let c = a.rotate_right(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 33
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 33;
    let c = a.rotate_right(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift 64 by 58
    let a = 64_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 58;
    let c = a.rotate_right(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);
}

// RANDOMIZED TESTS
// ================================================================================================

proptest! {
    #[test]
    fn unchecked_lt_proptest(a in any::<u64>(), b in any::<u64>()) {

        let (a1, a0) = split_u64(a);
        let (b1, b0) = split_u64(b);
        let c = (a < b) as u64;

        let source = "
            use.std::math::u64
            begin
                exec.u64::unchecked_lt
            end";

        build_test!(source, &[a0, a1, b0, b1]).prop_expect_stack(&[c])?;
    }

    #[test]
    fn unchecked_gt_proptest(a in any::<u64>(), b in any::<u64>()) {

        let (a1, a0) = split_u64(a);
        let (b1, b0) = split_u64(b);
        let c = (a > b) as u64;

        let source = "
            use.std::math::u64
            begin
                exec.u64::unchecked_gt
            end";

        build_test!(source, &[a0, a1, b0, b1]).prop_expect_stack(&[c])?;
    }

    #[test]
    fn unchecked_min_proptest(a in any::<u64>(), b in any::<u64>()) {

        let (a1, a0) = split_u64(a);
        let (b1, b0) = split_u64(b);
        let c = cmp::min(a, b) as u64;
        let (c1, c0) = split_u64(c);
        let source = "
            use.std::math::u64
            begin
                exec.u64::unchecked_min
            end";

        build_test!(source, &[a0, a1, b0, b1]).prop_expect_stack(&[c1, c0])?;
    }

    #[test]
    fn unchecked_max_proptest(a in any::<u64>(), b in any::<u64>()) {

        let (a1, a0) = split_u64(a);
        let (b1, b0) = split_u64(b);
        let c = cmp::max(a, b) as u64;
        let (c1, c0) = split_u64(c);
        let source = "
            use.std::math::u64
            begin
                exec.u64::unchecked_max
            end";

        build_test!(source, &[a0, a1, b0, b1]).prop_expect_stack(&[c1, c0])?;
    }

    #[test]
    fn unchecked_div_proptest(a in any::<u64>(), b in any::<u64>()) {

        let c = a / b;

        let (a1, a0) = split_u64(a);
        let (b1, b0) = split_u64(b);
        let (c1, c0) = split_u64(c);

        let source = "
            use.std::math::u64
            begin
                exec.u64::unchecked_div
            end";

        build_test!(source, &[a0, a1, b0, b1]).prop_expect_stack(&[c1, c0])?;
    }

    #[test]
    fn unchecked_mod_proptest(a in any::<u64>(), b in any::<u64>()) {

        let c = a % b;

        let (a1, a0) = split_u64(a);
        let (b1, b0) = split_u64(b);
        let (c1, c0) = split_u64(c);

        let source = "
            use.std::math::u64
            begin
                exec.u64::unchecked_mod
            end";

        build_test!(source, &[a0, a1, b0, b1]).prop_expect_stack(&[c1, c0])?;
    }

    fn shl_proptest(a in any::<u64>(), b in 0_u32..64) {

        let c = a.wrapping_shl(b);

        let (a1, a0) = split_u64(a);
        let (c1, c0) = split_u64(c);

        let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_shl
        end";

        build_test!(source, &[5, a0, a1, b as u64]).prop_expect_stack(&[c1, c0, 5])?;
    }

    #[test]
    fn shr_proptest(a in any::<u64>(), b in 0_u32..64) {

        let c = a.wrapping_shr(b);

        let (a1, a0) = split_u64(a);
        let (c1, c0) = split_u64(c);

        let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_shr
        end";

        build_test!(source, &[5, a0, a1, b as u64]).prop_expect_stack(&[c1, c0, 5])?;
    }

    #[test]
    fn rotl_proptest(a in any::<u64>(), b in 0_u32..64) {

        let c = a.rotate_left(b);

        let (a1, a0) = split_u64(a);
        let (c1, c0) = split_u64(c);

        let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_rotl
        end";

        build_test!(source, &[5, a0, a1, b as u64]).prop_expect_stack(&[c1, c0, 5])?;
    }

    #[test]
    fn rotr_proptest(a in any::<u64>(), b in 0_u32..64) {

        let c = a.rotate_right(b);

        let (a1, a0) = split_u64(a);
        let (c1, c0) = split_u64(c);

        let source = "
        use.std::math::u64
        begin
            exec.u64::unchecked_rotr
        end";

        build_test!(source, &[5, a0, a1, b as u64]).prop_expect_stack(&[c1, c0, 5])?;
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Split the provided u64 value into 32 high and low bits.
fn split_u64(value: u64) -> (u64, u64) {
    (value >> 32, value as u32 as u64)
}

fn split_u128(value: u128) -> (u64, u64, u64, u64) {
    (
        (value >> 96) as u64,
        (value >> 64) as u32 as u64,
        (value >> 32) as u32 as u64,
        value as u32 as u64,
    )
}
