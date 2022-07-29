use crate::build_op_test;
use crate::helpers::crypto::{init_merkle_leaf, init_merkle_leaves};
use rand_utils::rand_vector;
use vm_core::{AdviceSet, StarkField};

#[test]
fn rpperm() {
    let asm_op = "rpperm";
    let pub_inputs = rand_vector::<u64>(8);

    build_op_test!(asm_op, &pub_inputs).prove_and_verify(pub_inputs, 0, false);
}

#[test]
fn rphash() {
    let asm_op = "rphash";
    let pub_inputs = rand_vector::<u64>(8);

    build_op_test!(asm_op, &pub_inputs).prove_and_verify(pub_inputs, 0, false);
}

#[test]
fn mtree_get() {
    // drop's are added at the end to make sure stack overflow is empty on exit
    let asm_op = "mtree.get drop drop";

    let index = 3usize;
    let leaves = init_merkle_leaves(&[1, 2, 3, 4, 5, 6, 7, 8]);
    let tree = AdviceSet::new_merkle_tree(leaves).unwrap();

    let stack_inputs = [
        tree.root()[0].as_int(),
        tree.root()[1].as_int(),
        tree.root()[2].as_int(),
        tree.root()[3].as_int(),
        index as u64,
        tree.depth() as u64,
    ];

    build_op_test!(asm_op, &stack_inputs, &[], vec![tree]).prove_and_verify(
        stack_inputs.to_vec(),
        0,
        false,
    );
}

#[test]
fn mtree_set() {
    let asm_op = "mtree.set";
    let (stack_inputs, tree) = build_mtree_update_test_inputs();

    build_op_test!(asm_op, &stack_inputs, &[], vec![tree]).prove_and_verify(
        stack_inputs.to_vec(),
        0,
        false,
    );
}

#[test]
fn mtree_cwm() {
    // drop's are added at the end to make sure stack overflow is empty on exit
    let asm_op = "mtree.cwm drop drop";
    let (stack_inputs, tree) = build_mtree_update_test_inputs();

    build_op_test!(asm_op, &stack_inputs, &[], vec![tree]).prove_and_verify(stack_inputs, 0, false);
}

/// Helper function that builds a test stack and Merkle tree for testing mtree updates.
fn build_mtree_update_test_inputs() -> (Vec<u64>, AdviceSet) {
    let index = 5_usize;
    let leaves = init_merkle_leaves(&[1, 2, 3, 4, 5, 6, 7, 8]);
    let tree = AdviceSet::new_merkle_tree(leaves.clone()).unwrap();

    let new_node = init_merkle_leaf(9);
    let mut new_leaves = leaves;
    new_leaves[index] = new_node;

    let stack_inputs = vec![
        tree.root()[0].as_int(),
        tree.root()[1].as_int(),
        tree.root()[2].as_int(),
        tree.root()[3].as_int(),
        new_node[0].as_int(),
        new_node[1].as_int(),
        new_node[2].as_int(),
        new_node[3].as_int(),
        index as u64,
        tree.depth() as u64,
    ];

    (stack_inputs, tree)
}
