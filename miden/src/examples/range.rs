use crate::Example;
use miden::{assembly, Program, ProgramInputs};
use rand_utils::rand_vector;

// EXAMPLE BUILDER
// ================================================================================================

pub fn get_example(num_values: usize) -> Example {
    // generate random sequence of 64-bit values
    let values = generate_values(num_values);

    // generate the program and expected results
    let program = generate_range_check_program(num_values);
    let expected_result = vec![count_63_bit_values(&values)];
    println!(
        "Generated a program to range-check {} values; expected result: {}",
        num_values, expected_result[0]
    );

    Example {
        program,
        inputs: ProgramInputs::new(&[0], &values, &[]),
        pub_inputs: vec![0],
        expected_result,
        num_outputs: 1,
    }
}

/// Generates a random sequence of 64-bit values.
fn generate_values(n: usize) -> Vec<u128> {
    rand_vector::<u64>(n)
        .into_iter()
        .map(|v| v as u128)
        .collect()
}

/// Generates a program to range-check a sequence of values.
fn generate_range_check_program(n: usize) -> Program {
    let mut program = String::with_capacity(n * 80);
    program.push_str("begin ");

    // repeat the cycle of the following operations:
    // 1. read a value from secret tape A
    // 2. check if it fits into 63 bits (result is 1 if true, 0 otherwise)
    // 3. add the result into the running sum
    for _ in 0..n {
        program.push_str("read rc.63 add ");
    }
    program.push_str("end");

    assembly::compile(&program).unwrap()
}

/// Counts the number of values smaller than 63-bits in size.
fn count_63_bit_values(values: &[u128]) -> u128 {
    let p63 = 1 << 63;

    let mut result = 0;
    for &value in values.iter() {
        if value < p63 {
            result += 1;
        }
    }
    result
}

// EXAMPLE TESTER
// ================================================================================================

#[test]
fn test_range_example() {
    let example = get_example(20);
    super::test_example(example, false);
}

#[test]
fn test_range_example_fail() {
    let example = get_example(20);
    super::test_example(example, true);
}
