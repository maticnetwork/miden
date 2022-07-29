use rand_utils::rand_vector;
use vm_core::{
    hasher::{apply_permutation, hash_elements, STATE_WIDTH},
    AdviceSet, Felt, FieldElement, StarkField,
};

use crate::build_op_test;
use crate::helpers::crypto::{init_merkle_leaf, init_merkle_leaves};

// TESTS
// ================================================================================================

#[test]
fn rpperm() {
    let asm_op = "rpperm";

    // --- test hashing [ONE, ONE] ----------------------------------------------------------------
    let values: Vec<u64> = vec![2, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0];
    let expected = build_expected_perm(&values);

    let test = build_op_test!(asm_op, &values);
    let last_state = test.get_last_stack_state();

    assert_eq!(expected, &last_state[0..12]);

    // --- test hashing 8 random values -----------------------------------------------------------
    let mut values = rand_vector::<u64>(8);
    let capacity: Vec<u64> = vec![0, 0, 0, 8];
    values.extend_from_slice(&capacity);
    let expected = build_expected_perm(&values);

    let test = build_op_test!(asm_op, &values);
    let last_state = test.get_last_stack_state();

    assert_eq!(expected, &last_state[0..12]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let mut stack_inputs: Vec<u64> = vec![1, 2, 3, 4];
    let expected_stack_slice = stack_inputs
        .iter()
        .rev()
        .map(|&v| Felt::new(v))
        .collect::<Vec<Felt>>();

    let values_to_hash: Vec<u64> = vec![2, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0];
    stack_inputs.extend_from_slice(&values_to_hash);

    let test = build_op_test!(asm_op, &stack_inputs);
    let last_state = test.get_last_stack_state();

    assert_eq!(expected_stack_slice, &last_state[12..16]);
}

#[test]
fn rphash() {
    let asm_op = "rphash";

    // --- test hashing [ONE, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO] ----------------------------
    let values = [1, 1, 0, 0, 0, 0, 0, 0];
    let expected = build_expected_hash(&values);

    let test = build_op_test!(asm_op, &values);
    let last_state = test.get_last_stack_state();

    assert_eq!(expected, &last_state[..4]);

    // --- test hashing 8 random values -----------------------------------------------------------
    let values = rand_vector::<u64>(8);
    let expected = build_expected_hash(&values);

    let test = build_op_test!(asm_op, &values);
    let last_state = test.get_last_stack_state();

    assert_eq!(expected, &last_state[..4]);

    // --- test that the rest of the stack isn't affected -----------------------------------------
    let mut stack_inputs: Vec<u64> = vec![1, 2, 3, 4];
    let expected_stack_slice = stack_inputs
        .iter()
        .rev()
        .map(|&v| Felt::new(v))
        .collect::<Vec<Felt>>();

    let values_to_hash: Vec<u64> = vec![1, 1, 0, 0, 0, 0, 0, 0];
    stack_inputs.extend_from_slice(&values_to_hash);

    let test = build_op_test!(asm_op, &stack_inputs);
    let last_state = test.get_last_stack_state();

    assert_eq!(expected_stack_slice, &last_state[4..8]);
}

#[test]
fn mtree_get() {
    let asm_op = "mtree.get";

    let index = 3usize;
    let leaves = init_merkle_leaves(&[1, 2, 3, 4, 5, 6, 7, 8]);
    let tree = AdviceSet::new_merkle_tree(leaves.clone()).unwrap();

    let stack_inputs = [
        tree.root()[0].as_int(),
        tree.root()[1].as_int(),
        tree.root()[2].as_int(),
        tree.root()[3].as_int(),
        index as u64,
        tree.depth() as u64,
    ];

    let final_stack = [
        leaves[index][3].as_int(),
        leaves[index][2].as_int(),
        leaves[index][1].as_int(),
        leaves[index][0].as_int(),
        tree.root()[3].as_int(),
        tree.root()[2].as_int(),
        tree.root()[1].as_int(),
        tree.root()[0].as_int(),
    ];

    let test = build_op_test!(asm_op, &stack_inputs, &[], vec![tree]);
    test.expect_stack(&final_stack);
}

#[test]
fn mtree_update() {
    let index = 5usize;
    let leaves = init_merkle_leaves(&[1, 2, 3, 4, 5, 6, 7, 8]);
    let tree = AdviceSet::new_merkle_tree(leaves.clone()).unwrap();

    let new_node = init_merkle_leaf(9);
    let mut new_leaves = leaves;
    new_leaves[index] = new_node;
    let new_tree = AdviceSet::new_merkle_tree(new_leaves).unwrap();

    let stack_inputs = [
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

    // --- mtree.set ----------------------------------------------------------------------
    // update a node value and replace the old root
    let asm_op = "mtree.set";

    // expected state has the new leaf and the new root of the tree
    let final_stack = [
        new_node[3].as_int(),
        new_node[2].as_int(),
        new_node[1].as_int(),
        new_node[0].as_int(),
        new_tree.root()[3].as_int(),
        new_tree.root()[2].as_int(),
        new_tree.root()[1].as_int(),
        new_tree.root()[0].as_int(),
    ];

    let test = build_op_test!(asm_op, &stack_inputs, &[], vec![tree.clone()]);
    test.expect_stack(&final_stack);

    // --- mtree.cwm ----------------------------------------------------------------------
    // update a node value and replace the old root
    let asm_op = "mtree.cwm";

    // expected state has the new leaf, the new root of the tree, and the root of the old tree
    let final_stack = [
        new_node[3].as_int(),
        new_node[2].as_int(),
        new_node[1].as_int(),
        new_node[0].as_int(),
        new_tree.root()[3].as_int(),
        new_tree.root()[2].as_int(),
        new_tree.root()[1].as_int(),
        new_tree.root()[0].as_int(),
        tree.root()[3].as_int(),
        tree.root()[2].as_int(),
        tree.root()[1].as_int(),
        tree.root()[0].as_int(),
    ];

    let test = build_op_test!(asm_op, &stack_inputs, &[], vec![tree]);
    test.expect_stack(&final_stack);
}

// HELPER FUNCTIONS
// ================================================================================================

fn build_expected_perm(values: &[u64]) -> [Felt; STATE_WIDTH] {
    let mut expected = [Felt::ZERO; STATE_WIDTH];
    for (&value, result) in values.iter().zip(expected.iter_mut()) {
        *result = Felt::new(value);
    }
    apply_permutation(&mut expected);
    expected.reverse();

    expected
}

fn build_expected_hash(values: &[u64]) -> [Felt; 4] {
    let digest = hash_elements(&values.iter().map(|&v| Felt::new(v)).collect::<Vec<_>>());
    let mut expected: [Felt; 4] = digest.into();
    expected.reverse();

    expected
}
