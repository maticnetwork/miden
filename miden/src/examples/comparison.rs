use crate::Example;
use miden::{assembly, ProgramInputs};

// EXAMPLE BUILDER
// ================================================================================================

pub fn get_example(value: usize) -> Example {
    // determine the expected result
    let value = value as u128;
    let expected_result = if value < 9 { value * 9 } else { value + 9 };

    // construct the program which checks if the value provided via secret inputs is
    // less than 9; if it is, the value is multiplied by 9, otherwise, 9 is added
    // to the value; then we check if the value is odd.
    let program = assembly::compile(
        "
    begin
        push.9
        read
        dup.2
        lt.128
        if.true
            mul
        else
            add
        end
        dup
        isodd.128
    end",
    )
    .unwrap();

    println!(
        "Generated a program to test comparisons; expected result: {}",
        expected_result
    );

    Example {
        program,
        inputs: ProgramInputs::new(&[], &[value], &[]),
        pub_inputs: vec![],
        expected_result: vec![expected_result & 1, expected_result],
        num_outputs: 2,
    }
}

// EXAMPLE TESTER
// ================================================================================================

#[test]
fn test_comparison_example() {
    let example = get_example(10);
    super::test_example(example, false);
}

#[test]
fn test_comparison_example_fail() {
    let example = get_example(10);
    super::test_example(example, true);
}
