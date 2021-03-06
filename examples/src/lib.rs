use miden::{Program, ProgramInputs, ProofOptions};
use structopt::StructOpt;

pub mod collatz;
pub mod comparison;
pub mod conditional;
pub mod fibonacci;
#[cfg(feature = "std")]
pub mod merkle;
#[cfg(feature = "std")]
pub mod range;

// EXAMPLE
// ================================================================================================

pub struct Example {
    pub program: Program,
    pub inputs: ProgramInputs,
    pub pub_inputs: Vec<u128>,
    pub num_outputs: usize,
    pub expected_result: Vec<u128>,
}

// EXAMPLE OPTIONS
// ================================================================================================

#[derive(StructOpt, Debug)]
#[structopt(name = "Miden", about = "Miden examples")]
pub struct ExampleOptions {
    #[structopt(subcommand)]
    pub example: ExampleType,

    /// Security level for execution proofs generated by the VM
    #[structopt(short = "s", long = "security", default_value = "96bits")]
    security: String,
}

impl ExampleOptions {
    pub fn get_proof_options(&self) -> ProofOptions {
        match self.security.as_str() {
            "96bits" => ProofOptions::with_96_bit_security(),
            "128bits" => ProofOptions::with_128_bit_security(),
            other => panic!("{} is not a valid security level", other),
        }
    }
}

#[derive(StructOpt, Debug)]
//#[structopt(about = "available examples")]
pub enum ExampleType {
    /// Compute a Fibonacci sequence of the specified length
    Fib {
        /// Length of Fibonacci sequence
        #[structopt(short = "n", default_value = "1024")]
        sequence_length: usize,
    },
    /// Compute a Collatz sequence from the specified starting value
    Collatz {
        /// Starting value of the Collatz sequence
        #[structopt(short = "n", default_value = "511")]
        start_value: usize,
    },
    /// If provided value is less than 9, multiplies it by 9; otherwise add 9 to it
    Comparison {
        /// Value to compare to 9
        #[structopt(short = "n", default_value = "11")]
        value: usize,
    },
    /// If provided value is 0, outputs 15; if provided value is 1, outputs 8
    Conditional {
        /// Value to compare to 9
        #[structopt(short = "n", default_value = "1")]
        value: usize,
    },
    /// Computes a root of a randomly generated Merkle branch of the specified depth
    #[cfg(feature = "std")]
    Merkle {
        /// Depth of the Merkle tree
        #[structopt(short = "n", default_value = "20")]
        tree_depth: usize,
    },
    /// Determines how many of the randomly generated values are less than 2^63
    #[cfg(feature = "std")]
    Range {
        /// Number of randomly generated 64-bit values
        #[structopt(short = "n", default_value = "100")]
        num_values: usize,
    },
}

// TESTS
// ================================================================================================

#[cfg(test)]
pub fn test_example(example: Example, fail: bool) {
    let Example {
        program,
        inputs,
        pub_inputs,
        num_outputs,
        expected_result,
    } = example;

    let options = ProofOptions::new(
        32,
        8,
        0,
        miden::HashFunction::Blake3_256,
        miden::FieldExtension::None,
        8,
        256,
    );

    let (mut outputs, proof) = miden::execute(&program, &inputs, num_outputs, &options).unwrap();

    assert_eq!(
        expected_result, outputs,
        "Program result was computed incorrectly"
    );

    if fail {
        outputs[0] = outputs[0] + 1;
        assert!(miden::verify(*program.hash(), &pub_inputs, &outputs, proof).is_err())
    } else {
        assert!(miden::verify(*program.hash(), &pub_inputs, &outputs, proof).is_ok());
    }
}
