use crate::{build_op_test, build_test};

/// Range checks the result of 1 + 1. This results in 2 range checks, one for each 16-bit limb of
/// the 32-bit result (2 and 0).
#[test]
fn range_check_once() {
    let asm_op = "u32overflowing_add";
    let stack = vec![1, 1];

    build_op_test!(asm_op, &stack).prove_and_verify(stack, 0, false);
}

/// Range checks multiple values a varying number of times, since each value is checked as an input.
/// 5 is checked 3 times, 10 is checked twice, and 15 is checked once.
#[test]
fn range_check_multi() {
    let source = "begin u32checked_add u32checked_add end";
    let stack = vec![5, 5, 5];
    build_test!(source, &stack).prove_and_verify(stack, 0, false);
}
