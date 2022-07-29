use crate::Example;
use miden::{assembly, ProgramInputs};

// EXAMPLE BUILDER
// ================================================================================================

pub fn get_example(flag: usize) -> Example {
    // convert flag to a field element
    let flag = flag as u128;

    // determine the expected result
    let expected_result = match flag {
        0 => 15u128,
        1 => 8u128,
        _ => panic!("flag must be a binary value"),
    };

    // construct the program which either adds or multiplies two numbers
    // based on the value provided via secret inputs
    let program = assembly::compile(
        "
    begin
        push.3
        push.5
        read
        if.true
            add
        else
            mul
        end
    end",
    )
    .unwrap();

    println!(
        "Generated a program to test conditional execution; expected result: {}",
        expected_result
    );

    Example {
        program,
        inputs: ProgramInputs::new(&[], &[flag], &[]),
        pub_inputs: vec![],
        expected_result: vec![expected_result],
        num_outputs: 1,
    }
}

// EXAMPLE TESTER
// ================================================================================================

#[test]
fn test_conditional_example() {
    let example = get_example(1);
    super::test_example(example, false);
}

#[test]
fn test_conditional_example_fail() {
    let example = get_example(1);
    super::test_example(example, true);
}
