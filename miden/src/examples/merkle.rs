use crate::Example;
use miden::{assembly, BaseElement, FieldElement, Program, ProgramInputs, StarkField};
use rand_utils::prng_vector;
use vm_core::hasher;

// EXAMPLE BUILDER
// ================================================================================================

pub fn get_example(depth: usize) -> Example {
    assert!(
        depth >= 2,
        "tree depth must be at least 2, but received {}",
        depth
    );

    // generate a pseudo-random Merkle authentication path
    let (auth_path, leaf_index) = generate_authentication_path(depth);

    // compute root of the Merkle tree to which the path resolves
    let mut expected_result = compute_merkle_root(&auth_path, leaf_index);
    println!("Expected tree root: {:?}", expected_result);

    // generate the program to verify Merkle path of given length
    let program = generate_merkle_program(depth, leaf_index);
    println!(
        "Generated a program to verify Merkle proof for a tree of depth {}",
        depth
    );

    // double and reverse tree root because values on the stack are in reverse order
    expected_result.push(expected_result[0]);
    expected_result.push(expected_result[1]);
    expected_result.reverse();

    Example {
        program,
        inputs: generate_program_inputs(&auth_path, leaf_index),
        pub_inputs: vec![],
        expected_result,
        num_outputs: 4,
    }
}

/// Returns a program to verify Merkle authentication paths for a tree of depth `n`;
/// the program first verifies the path using smpath operation, and then verifies
/// the same path using pmpath operation.
fn generate_merkle_program(n: usize, index: usize) -> Program {
    let source = format!(
        "
    begin
        read.ab
        dup.2
        smpath.{}
        swap.2
        push.{}
        roll.4 swap swap.2
        pmpath.{}
    end
    ",
        n, index, n
    );

    assembly::compile(&source).unwrap()
}

/// Converts Merkle authentication path for a node at the specified `index` into
/// a set of inputs which can be consumed by the program created by the function above.
fn generate_program_inputs(path: &[Vec<BaseElement>; 2], index: usize) -> ProgramInputs {
    let mut a = Vec::new();
    let mut b = Vec::new();
    let n = path[0].len();
    let mut index = index + usize::pow(2, (n - 1) as u32);

    // push the leaf node onto secret input tapes A and B
    a.push(path[0][0].as_int());
    b.push(path[1][0].as_int());

    // populate the tapes with inputs for smpath operation
    for i in 1..n {
        // push next bit of the position index onto tapes A and B; we use both tapes
        // here so that we can use READ2 instruction when reading inputs from the tapes
        a.push(0);
        b.push((index & 1) as u128);
        index >>= 1;

        // push the next node onto tapes A and B
        a.push(path[0][i].as_int());
        b.push(path[1][i].as_int());
    }

    // populate the tapes with inputs for pmpath operation
    for i in 1..n {
        a.push(path[0][i].as_int());
        b.push(path[1][i].as_int());
    }

    ProgramInputs::new(&[], &a, &b)
}

/// Pseudo-randomly generates a Merkle authentication path for an imaginary Merkle tree
/// of depth equal to `n`
fn generate_authentication_path(n: usize) -> ([Vec<BaseElement>; 2], usize) {
    let mut s1 = [0u8; 32];
    s1[0] = 1;
    s1[1] = 2;
    s1[2] = 3;
    let mut s2 = [0u8; 32];
    s2[0] = 4;
    s2[1] = 5;
    s2[2] = 6;

    let leaves = u128::pow(2, (n - 1) as u32);
    let leaf_index = (prng_vector::<BaseElement>(s1, 1)[0].as_int() % leaves) as usize;

    ([prng_vector(s1, n), prng_vector(s2, n)], leaf_index)
}

/// Computes tree root to which a given authentication path resolves assuming the
/// path is for a leaf node at position specified by `index` parameter.
fn compute_merkle_root(path: &[Vec<BaseElement>; 2], index: usize) -> Vec<u128> {
    let mut buf = [BaseElement::ZERO; 4];
    let n = path[0].len();

    let r = index & 1;
    buf[0] = path[0][r];
    buf[1] = path[1][r];
    buf[2] = path[0][1 - r];
    buf[3] = path[1][1 - r];

    let mut v = hasher::digest(&buf);

    let mut index = (index + usize::pow(2, (n - 1) as u32)) >> 1;
    for i in 2..n {
        if index & 1 == 0 {
            buf[0] = v[0];
            buf[1] = v[1];
            buf[2] = path[0][i];
            buf[3] = path[1][i];
        } else {
            buf[0] = path[0][i];
            buf[1] = path[1][i];
            buf[2] = v[0];
            buf[3] = v[1];
        }

        v = hasher::digest(&buf);
        index >>= 1;
    }

    v.iter().map(|e| e.as_int()).collect()
}

// EXAMPLE TESTER
// ================================================================================================

#[test]
fn test_merkle_example() {
    let example = get_example(4);
    super::test_example(example, false);
}

#[test]
fn test_merkle_example_fail() {
    let example = get_example(4);
    super::test_example(example, true);
}
