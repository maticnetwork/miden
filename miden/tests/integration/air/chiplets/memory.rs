use crate::{build_op_test, build_test};
use vm_core::utils::ToElements;

#[test]
fn push() {
    // drop's are added at the end to make sure stack overflow is empty on exit
    let asm_op = "push.mem.0 swap drop";

    build_op_test!(asm_op).prove_and_verify(vec![], 0, false);
}

#[test]
fn pushw() {
    // drop's are added at the end to make sure stack overflow is empty on exit
    let asm_op = "pushw.mem.0 swapw drop drop drop drop";

    build_op_test!(asm_op).prove_and_verify(vec![], 0, false);
}

#[test]
fn pop() {
    let asm_op = "pop.mem.0";
    let pub_inputs = vec![1];

    build_op_test!(asm_op, &pub_inputs).prove_and_verify(pub_inputs, 0, false);
}

#[test]
fn helper_pop() {
    // Sequence of operations: [Span, Pad, MStore, Drop, Pad, Mstore, Drop, Pad, Mstore, Drop]
    let asm_op = "begin pop.mem.0 pop.mem.0 pop.mem.0 end";
    let pub_inputs = vec![1, 2];

    let trace = build_test!(asm_op, &pub_inputs).execute().unwrap();
    // Since the MStore operation stores in helper registers the value that was previously in
    // memory, after the first call to MStore, the helper registers will be filled with zeros
    // (since memory is initialized with zeros by default). And an element taken from the
    // top of the stack, that is, 2, will be written to memory. On the second call, the memory will
    // be overwritten by 1, and the previous value that was in memory, that is, 2, will be written
    // to the helper registers.
    let helper_regs = [2, 0, 0, 0, 0, 0].to_elements();
    // We need to check helper registers state after second MStore, which index is 5
    assert_eq!(helper_regs, trace.get_user_op_helpers_at(5));
    // The next time the MStore operation is called, the memory will be overwritten again, and the
    // 1 lying there before that will be written to the helper register
    let helper_regs = [1, 0, 0, 0, 0, 0].to_elements();
    // We need to check helper registers state after third MStore, which index is 8
    assert_eq!(helper_regs, trace.get_user_op_helpers_at(8));
}

#[test]
fn popw() {
    let asm_op = "popw.mem.0";
    let pub_inputs = vec![1, 2, 3, 4];

    build_op_test!(asm_op, &pub_inputs).prove_and_verify(pub_inputs, 0, false);
}

#[test]
fn helper_popow() {
    // Sequence of operations: [Span, Pad, MStorew, Drop, Drop, Drop, Drop, Pad, Mstore, Drop,
    //                          Drop, Drop, Drop, Pad, Mstore, Drop, Drop, Drop, Drop]
    let asm_op = "begin popw.mem.0 popw.mem.0 popw.mem.0 end";
    let pub_inputs = vec![213, 214, 215, 216, 217, 218, 219, 220];

    let trace = build_test!(asm_op, &pub_inputs).execute().unwrap();
    // Filling in helper registers is similar to the helper_pop test, with the difference that not
    // one element is written to the registers, but the whole word
    let helper_regs1 = [217, 218, 219, 220, 0, 0].to_elements();
    // We need to check helper registers state after second MStore, which index is 8
    assert_eq!(helper_regs1, trace.get_user_op_helpers_at(8));
    let helper_regs1 = [213, 214, 215, 216, 0, 0].to_elements();
    // We need to check helper registers state after second MStore, which index is 14
    assert_eq!(helper_regs1, trace.get_user_op_helpers_at(14));
}

#[test]
fn load() {
    let asm_op = "loadw.mem.0";

    build_op_test!(asm_op).prove_and_verify(vec![], 0, false);
}

#[test]
fn store() {
    let asm_op = "storew.mem.0";
    let pub_inputs = vec![1, 2, 3, 4];

    build_op_test!(asm_op, &pub_inputs).prove_and_verify(pub_inputs, 0, false);
}

#[test]
fn write_read() {
    // drop's are added at the end to make sure stack overflow is empty on exit
    let source = "begin popw.mem.0 pushw.mem.0 swapw drop drop drop drop end";
    let pub_inputs = vec![4, 3, 2, 1];

    build_test!(source, &pub_inputs).prove_and_verify(pub_inputs, 1, false);
}

#[test]
fn helper_write_read() {
    // drop's are added at the end to make sure stack overflow is empty on exit
    // Sequence of operations: [Span, Pad, MStorew, Drop, Drop, Drop, Drop, Pad, MLoad, ... ]
    let source = "begin popw.mem.0 push.mem.0 swapw drop end";
    let pub_inputs = vec![4, 3, 2, 1];

    let trace = build_test!(source, &pub_inputs).execute().unwrap();
    // When the MLoad operation is called, word elements that were not pushed on the stack
    // are written to helper registers. So, 3, 2 and 1 will be written after this operation
    let helper_regs = [3, 2, 1, 0, 0, 0].to_elements();
    // We need to check helper registers state after first MLoad, which index is 8
    assert_eq!(helper_regs, trace.get_user_op_helpers_at(8));
}

#[test]
fn update() {
    // drop's are added at the end to make sure stack overflow is empty on exit
    let source = "begin pushw.mem.0 storew.mem.0 swapw drop drop drop drop end";
    let pub_inputs = vec![8, 7, 6, 5, 4, 3, 2, 1];

    build_test!(source, &pub_inputs).prove_and_verify(pub_inputs, 1, false);
}

#[test]
fn incr_write_addr() {
    let source = "begin storew.mem.0 storew.mem.1 end";
    let pub_inputs = vec![4, 3, 2, 1];

    build_test!(source, &pub_inputs).prove_and_verify(pub_inputs, 1, false);
}
