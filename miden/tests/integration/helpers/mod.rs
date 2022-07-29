pub use miden::{ProofOptions, StarkProof};
use processor::{ExecutionError, ExecutionTrace, Process, VmStateIterator};
use proptest::prelude::*;
pub use vm_core::{Felt, FieldElement, Program, ProgramInputs, MIN_STACK_DEPTH};

pub mod crypto;

// CONSTANTS
// ================================================================================================
pub const WORD_LEN: usize = 4;

// TEST HANDLER
// ================================================================================================

/// This is used to specify the expected error type when using Test to test errors.
/// `Test::expect_error` will try to either compile or execute the test data, according to the
/// provided TestError variant. Then it will validate that the resulting error contains the
/// TestError variant's string slice.
pub enum TestError<'a> {
    AssemblyError(&'a str),
    ExecutionError(&'a str),
}

/// This is a container for the data required to run tests, which allows for running several
/// different types of tests.
///
/// Types of valid result tests:
/// - Execution test: check that running a program compiled from the given source has the
///   specified results for the given (optional) inputs.
/// - Proptest: run an execution test inside a proptest.
///
/// Types of failure tests:
/// - Assembly error test: check that attempting to compile the given source causes an
/// AssemblyError which contains the specified substring.
/// - Execution error test: check that running a program compiled from the given source causes
///   an ExecutionError which contains the specified substring.
pub struct Test {
    pub source: String,
    pub inputs: ProgramInputs,
    pub in_debug_mode: bool,
}

impl Test {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------

    /// Creates the simplest possible new test, with only a source string and no inputs.
    pub fn new(source: &str, in_debug_mode: bool) -> Self {
        Test {
            source: String::from(source),
            inputs: ProgramInputs::none(),
            in_debug_mode,
        }
    }

    // TEST METHODS
    // --------------------------------------------------------------------------------------------

    /// Asserts that running the test for the expected TestError variant will result in an error
    /// that contains the TestError's error substring in its error message.
    pub fn expect_error(&self, error: TestError) {
        match error {
            TestError::AssemblyError(substr) => {
                assert_eq!(
                    std::panic::catch_unwind(|| self.compile())
                        .err()
                        .and_then(|a| { a.downcast_ref::<String>().map(|s| s.contains(substr)) }),
                    Some(true)
                );
            }
            TestError::ExecutionError(substr) => {
                assert_eq!(
                    std::panic::catch_unwind(|| self.execute().unwrap())
                        .err()
                        .and_then(|a| { a.downcast_ref::<String>().map(|s| s.contains(substr)) }),
                    Some(true)
                );
            }
        }
    }

    /// Builds a final stack from the provided stack-ordered array and asserts that executing the
    /// test will result in the expected final stack state.
    pub fn expect_stack(&self, final_stack: &[u64]) {
        let expected = convert_to_stack(final_stack);
        let result = self.get_last_stack_state();

        assert_eq!(expected, result);
    }

    /// Executes the test and validates that the process memory has the elements of `expected_mem`
    /// at address `mem_addr` and that the end of the stack execution trace matches the
    /// `final_stack`.
    pub fn expect_stack_and_memory(
        &self,
        final_stack: &[u64],
        mem_addr: u64,
        expected_mem: &[u64],
    ) {
        let mut process = Process::new(self.inputs.clone());

        // execute the test
        let program = self.compile();
        process.execute(&program).unwrap();

        // validate the memory state
        let mem_state = process.get_memory_value(mem_addr).unwrap();
        let expected_mem: Vec<Felt> = expected_mem.iter().map(|&v| Felt::new(v)).collect();
        assert_eq!(expected_mem, mem_state);

        // validate the stack state
        self.expect_stack(final_stack);
    }

    /// Asserts that executing the test inside a proptest results in the expected final stack state.
    /// The proptest will return a test failure instead of panicking if the assertion condition
    /// fails.
    pub fn prop_expect_stack(
        &self,
        final_stack: &[u64],
    ) -> Result<(), proptest::test_runner::TestCaseError> {
        let expected = convert_to_stack(final_stack);
        let result = self.get_last_stack_state();

        prop_assert_eq!(expected, result);

        Ok(())
    }

    // UTILITY METHODS
    // --------------------------------------------------------------------------------------------

    /// Compiles a test's source and returns the resulting Program.
    pub fn compile(&self) -> Program {
        let assembler = assembly::Assembler::new(self.in_debug_mode);
        assembler
            .compile(&self.source)
            .expect("Failed to compile test source.")
    }

    /// Compiles the test's source to a Program and executes it with the tests inputs. Returns a
    /// resulting execution trace or error.
    pub fn execute(&self) -> Result<ExecutionTrace, ExecutionError> {
        let program = self.compile();
        processor::execute(&program, &self.inputs)
    }

    /// Compiles the test's code into a program, then generates and verifies a proof of execution
    /// using the given public inputs and the specified number of stack outputs. When `test_fail`
    /// is true, this function will force a failure by modifying the first output.
    pub fn prove_and_verify(
        &self,
        pub_inputs: Vec<u64>,
        num_stack_outputs: usize,
        test_fail: bool,
    ) {
        let program = self.compile();
        let (mut outputs, proof) = prover::prove(
            &program,
            &self.inputs,
            num_stack_outputs,
            &ProofOptions::default(),
        )
        .unwrap();

        if test_fail {
            outputs[0] += 1;
            assert!(miden::verify(program.hash(), &pub_inputs, &outputs, proof).is_err());
        } else {
            assert!(miden::verify(program.hash(), &pub_inputs, &outputs, proof).is_ok());
        }
    }

    /// Compiles the test's source to a Program and executes it with the tests inputs. Returns a
    /// VmStateIterator that allows us to iterate through each clock cycle and inpsect the process
    /// state.
    pub fn execute_iter(&self) -> VmStateIterator {
        let program = self.compile();
        processor::execute_iter(&program, &self.inputs)
    }

    /// Returns the last state of the stack after executing a test.
    pub fn get_last_stack_state(&self) -> [Felt; MIN_STACK_DEPTH] {
        let trace = self.execute().unwrap();

        trace.last_stack_state()
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Takes an array of u64 values and builds a stack, perserving their order and converting them to
/// field elements.
pub fn convert_to_stack(values: &[u64]) -> [Felt; MIN_STACK_DEPTH] {
    let mut result = [Felt::ZERO; MIN_STACK_DEPTH];
    for (&value, result) in values.iter().zip(result.iter_mut()) {
        *result = Felt::new(value);
    }
    result
}

// This is a proptest strategy for generating a random word with 4 values of type T.
pub fn prop_randw<T: proptest::arbitrary::Arbitrary>() -> impl Strategy<Value = Vec<T>> {
    prop::collection::vec(any::<T>(), 4)
}
