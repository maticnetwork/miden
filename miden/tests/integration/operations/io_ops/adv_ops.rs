use super::{build_op_test, TestError};

// PUSHING VALUES ONTO THE STACK (PUSH)
// ================================================================================================

#[test]
fn push_adv() {
    let asm_op = "push.adv";
    let advice_tape = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let test_n = |n: usize| {
        let source = format!("{}.{}", asm_op, n);
        let mut final_stack = vec![0; n];
        final_stack.copy_from_slice(&advice_tape[..n]);
        final_stack.reverse();

        let test = build_op_test!(source, &[], &advice_tape, vec![]);
        test.expect_stack(&final_stack);
    };

    // --- push 1 ---------------------------------------------------------------------------------
    test_n(1);

    // --- push max -------------------------------------------------------------------------------
    test_n(16);
}

#[test]
fn push_adv_invalid() {
    // attempting to read from empty advice tape should throw an error
    let test = build_op_test!("push.adv.1");
    test.expect_error(TestError::ExecutionError("EmptyAdviceTape"));
}

// OVERWRITING VALUES ON THE STACK (LOAD)
// ================================================================================================

#[test]
fn loadw_adv() {
    let asm_op = "loadw.adv";
    let advice_tape = [1, 2, 3, 4];
    let mut final_stack = advice_tape;
    final_stack.reverse();

    let test = build_op_test!(asm_op, &[8, 7, 6, 5], &advice_tape, vec![]);
    test.expect_stack(&final_stack);
}

#[test]
fn loadw_adv_invalid() {
    // attempting to read from empty advice tape should throw an error
    let test = build_op_test!("loadw.adv", &[0, 0, 0, 0]);
    test.expect_error(TestError::ExecutionError("EmptyAdviceTape"));
}
