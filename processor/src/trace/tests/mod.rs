use super::{ExecutionTrace, Felt, FieldElement, LookupTableRow, Process, Trace, NUM_RAND_ROWS};
use rand_utils::rand_array;
use vm_core::{code_blocks::CodeBlock, Operation, ProgramInputs, Word, ONE, ZERO};

mod chiplets;
mod hasher;
mod range;
mod stack;

// TEST HELPERS
// ================================================================================================

/// Builds a sample trace by executing the provided code block against the provided stack inputs.
pub fn build_trace_from_block(program: &CodeBlock, stack: &[u64]) -> ExecutionTrace {
    let inputs = ProgramInputs::new(stack, &[], vec![]).unwrap();
    let mut process = Process::new(inputs);
    process.execute_code_block(program).unwrap();
    ExecutionTrace::new(process)
}

/// Builds a sample trace by executing a span block containing the specified operations. This
/// results in 1 additional hash cycle (8 rows) at the beginning of the hash chiplet.
pub fn build_trace_from_ops(operations: Vec<Operation>, stack: &[u64]) -> ExecutionTrace {
    let program = CodeBlock::new_span(operations);
    build_trace_from_block(&program, stack)
}

/// Builds a sample trace by executing a span block containing the specified operations. Unlike the
/// function above, this function accepts the full [ProgramInputs] object, which means it can run
/// the programs with initialized advice provider.
pub fn build_trace_from_ops_with_inputs(
    operations: Vec<Operation>,
    inputs: ProgramInputs,
) -> ExecutionTrace {
    let mut process = Process::new(inputs);
    let program = CodeBlock::new_span(operations);
    process.execute_code_block(&program).unwrap();
    ExecutionTrace::new(process)
}
