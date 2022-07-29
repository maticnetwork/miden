use super::{
    super::validate_operation, parse_decimal_param, parse_element_param, parse_hex_param,
    parse_u32_param, push_value, AssemblyError, Felt, Operation, Token, Vec,
};
use vm_core::{AdviceInjector, Decorator, DecoratorList};

mod adv_ops;
mod constant_ops;
mod env_ops;
mod local_ops;
mod mem_ops;

use adv_ops::*;
use constant_ops::*;
use env_ops::*;
use local_ops::*;
use mem_ops::*;

// PUSHING VALUES ONTO THE STACK (PUSH)
// ================================================================================================

/// Pushes constant, environment, memory, or non-deterministic (advice) inputs onto the stack as
/// specified by the operation variant and its parameter(s).
///
/// *CONSTANTS: `push.a`, `push.a.b`, `push.a.b.c...`*
/// Pushes immediate values `a`, `b`, `c`, etc onto the stack in the order in which they're
/// provided. Up to 16 values can be specified. All values must be valid field elements in decimal
/// (e.g. 123) or hexadecimal (e.g. 0x7b) representation. When specifying values in hexadecimal
/// format, it is possible to omit the periods between individual values as long as the total number
/// of specified bytes is a multiple of 8.
///
/// *NON-DETERMINISTIC (ADVICE): `push.adv.n`*
/// Removes the next `n` values from the advice tape and pushes them onto the stack. The number of
/// items that can be read from the advice tape is limited to 16.
///
/// *ENVIRONMENT: `push.env.{var}`*
/// Pushes the value of the specified environment variable onto the top of the stack. Currently, the
/// only environment input is `sdepth`.
///
/// *RANDOM ACCESS MEMORY: `push.mem`, `push.mem.a`*
/// Pushes the first element of the word at memory address `a` onto the stack. If no memory address
/// is specified, it is assumed to be on top of the stack.
///
/// *LOCAL PROCEDURE VARIABLES: `push.local.i`*
/// Pushes the first element of the word at the local memory address with index `i` onto the stack.
///
/// # Errors
///
/// Returns an `AssemblyError` if the op param is invalid, malformed, or doesn't match an expected
/// `push` instruction
pub fn parse_push(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    num_proc_locals: u32,
) -> Result<(), AssemblyError> {
    if op.num_parts() < 2 {
        return Err(AssemblyError::invalid_op(op));
    }
    if op.parts()[0] != "push" {
        return Err(AssemblyError::unexpected_token(
            op,
            "push.{adv.n|env.var|local.i|mem|mem.a|a|a.b|a.b.c...}",
        ));
    }

    match op.parts()[1] {
        "adv" => parse_push_adv(span_ops, op),
        "env" => parse_push_env(span_ops, op, num_proc_locals),
        "local" => parse_push_local(span_ops, op, num_proc_locals),
        "mem" => parse_push_mem(span_ops, op),
        _ => parse_push_constant(span_ops, op),
    }
}

/// Pushes a word (4 elements) onto the stack from an absolute location in random access memory or
/// from local procedure memory as specified by the operation variant and its parameter.
///
/// *RANDOM ACCESS MEMORY: `pushw.mem`, `pushw.mem.a`*
/// Reads a word (4 elements) from memory and pushes it onto the stack by appending `LOADW` and
/// required stack manipulations to the span block. If no memory address is specified, it is assumed
/// to be on top of the stack. Otherwise, the provided address will be pushed so it is on top of the
/// stack when `LOADW` is executed. The memory address will be removed from the stack by `LOADW`.
///
/// *LOCAL PROCEDURE VARIABLES: `pushw.local.i`*
/// Reads a word (4 elements) from local memory at index `i` and pushes it onto the stack.
///
/// # Errors
///
/// Returns an `AssemblyError` if the op param is invalid, malformed, or doesn't match an expected
/// `pushw` instruction.
pub fn parse_pushw(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    num_proc_locals: u32,
) -> Result<(), AssemblyError> {
    validate_operation!(op, "pushw.local|mem");

    match op.parts()[1] {
        // read from mem with overwrite_stack_top set to false so the rest of the stack is kept
        "mem" => parse_read_mem(span_ops, op, false),
        "local" => parse_read_local(span_ops, op, num_proc_locals, false),
        _ => Err(AssemblyError::invalid_op(op)),
    }
}

// REMOVING VALUES FROM THE STACK (POP)
// ================================================================================================

/// Pops an element off the stack and saves it to the absolute or local memory location as
/// specified by the operation variant and its parameter.
///
/// *RANDOM ACCESS MEMORY: `pop.mem`, `pop.mem.a`*
/// Pops an element off the stack and saves it at memory address `a`. If no memory address is
/// specified as part of the operation, then it is assumed to be on top of the stack.
///
/// *LOCAL PROCEDURE VARIABLES: `pop.local.i`*
/// Pops an element off the stack and saves it to the local memory address with index `i`.
///
/// # Errors
///
/// Returns an `AssemblyError` if the op param is invalid, malformed, or doesn't match an expected
/// `pop` instruction
pub fn parse_pop(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    num_proc_locals: u32,
) -> Result<(), AssemblyError> {
    validate_operation!(op, "pop.local|mem");

    match op.parts()[1] {
        "local" => parse_pop_local(span_ops, op, num_proc_locals),
        "mem" => parse_pop_mem(span_ops, op),
        _ => Err(AssemblyError::invalid_op(op)),
    }
}

/// Pops a word (4 elements) from the stack and store it at an absolute memory location or in local
/// procedure memory as specified by the operation variant and its parameter.
///
/// *RANDOM ACCESS MEMORY: `popw.mem`, `popw.mem.a`*
/// Pops the top 4 elements off the stack and stores them at an absolute address in memory by
/// appending `STOREW` and required stack manipulations to the span block. If no memory address is
/// provided as a parameter, the address is assumed to be on top of the stack. Otherwise, the
/// provided address will be pushed so it is on top of the stack when `STOREW` is executed. The
/// memory address will be removed from the stack by `STOREW`.
///
/// *LOCAL PROCEDURE VARIABLES: `popw.local.i`*
/// Pops the top 4 elements off the stack and stores them in local memory at index `i`.
///
/// # Errors
///
/// Returns an `AssemblyError` if the op param is invalid, malformed, or doesn't match an expected
/// `popw` instruction.
pub fn parse_popw(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    num_proc_locals: u32,
) -> Result<(), AssemblyError> {
    validate_operation!(op, "popw.local|mem");

    match op.parts()[1] {
        // write to mem with retain_stack_top set to false so the 4 elements are dropped after writing
        "mem" => parse_write_mem(span_ops, op, false),
        "local" => parse_write_local(span_ops, op, num_proc_locals, false),
        _ => Err(AssemblyError::invalid_op(op)),
    }
}

// OVERWRITING VALUES ON THE STACK (LOAD)
// ================================================================================================

/// Overwrites the top 4 elements of the stack with a word (4 elements) loaded from either the
/// advice tape, an absolute location in random access memory, or procedure locals as specified by
/// the operation variant and its parameter.
///
/// *NON-DETERMINISTIC (ADVICE): `loadw.adv`*
/// Removes the next word (4 elements) from the advice tape and overwrites the top 4 elements of the
/// stack with it. Fails if the advice tape has fewer than 4 elements.
///
/// *RANDOM ACCESS MEMORY: `loadw.mem`, `loadw.mem.a`*
/// Reads a word (4 elements) from memory and overwrites the top 4 elements of the stack with it by
/// appending `LOADW` and required stack manipulations to the span block. If no memory address is
/// specified, the address is assumed to be on top of the stack. Otherwise, the provided address
/// will be pushed so it is on top of the stack when `LOADW` is executed. The memory address will be
/// removed from the stack by `LOADW`.
///
/// *LOCAL PROCEDURE VARIABLES: `loadw.local.i`*
/// Reads a word (4 elements) from local memory at index `i` and overwrites the top 4 elements of
/// the stack with it.
///
/// # Errors
///
/// Returns an `AssemblyError` if the op param is invalid, malformed, or doesn't match an expected
/// `loadw` instruction.
pub fn parse_loadw(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    num_proc_locals: u32,
) -> Result<(), AssemblyError> {
    validate_operation!(op, "loadw.adv|local|mem");

    match op.parts()[1] {
        "adv" => parse_loadw_adv(span_ops, op),
        // read from mem with overwrite_stack_top set to true so the top 4 elements are overwritten
        "mem" => parse_read_mem(span_ops, op, true),
        "local" => parse_read_local(span_ops, op, num_proc_locals, true),
        _ => Err(AssemblyError::invalid_op(op)),
    }
}

// SAVING STACK VALUES WITHOUT REMOVING THEM (STORE)
// ================================================================================================

/// Stores the top 4 elements of the stack at an absolute memory location or in local procedure
/// memory, as specified by the operation variant and its parameter. If a memory address is provided
/// via the stack, it will be removed first. At the end of the operation, all elements will remain
/// on the stack.
///
/// *RANDOM ACCESS MEMORY: `storew.mem`, `storew.mem.a`*
/// Stores the top 4 elements of the stack at an absolute address in memory by appending `STOREW`
/// and required stack manipulations to the span block. If no memory address is provided as a
/// parameter, the address is assumed to be on top of the stack. Otherwise, the provided address
/// will be pushed so it is on top of the stack when `STOREW` is executed.  The memory address will
/// be removed from the stack by `STOREW`.
///
/// *LOCAL PROCEDURE VARIABLES: `storew.local.i`*
/// Stores the top 4 elements of the stack in local memory at index `i`.
///
/// # Errors
///
/// Returns an `AssemblyError` if the op param is invalid, malformed, or doesn't match an expected
/// `storew` instruction.
pub fn parse_storew(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    num_proc_locals: u32,
) -> Result<(), AssemblyError> {
    validate_operation!(op, "storew.local|mem");

    match op.parts()[1] {
        // write to mem with retain_stack_top set to true so the 4 elements are left on the stack
        "mem" => parse_write_mem(span_ops, op, true),
        "local" => parse_write_local(span_ops, op, num_proc_locals, true),
        _ => Err(AssemblyError::invalid_op(op)),
    }
}

// ADVICE INJECTORS
// ================================================================================================

/// Appends the appropriate advice injector operation to `span_ops`.
///
/// Advice injector operations insert one or more values at the head of the advice tape, but do
/// not modify the VM state and do not advance the clock cycles. Currently, the following advice
/// injectors can be invoked explicitly:
/// - adv.u64div: this operation interprets four elements at the top of the stack as two 64-bit
///   values (represented by 32-bit limbs), divides one value by another, and injects the quotient
///   and the remainder into the advice tape.
pub fn parse_adv_inject(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    decorators: &mut DecoratorList,
) -> Result<(), AssemblyError> {
    validate_operation!(op, "adv.u64div");
    match op.parts()[1] {
        "u64div" => decorators.push((
            span_ops.len(),
            Decorator::Advice(AdviceInjector::DivResultU64),
        )),
        _ => return Err(AssemblyError::invalid_op(op)),
    };

    Ok(())
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{
        parse_loadw, parse_pop, parse_popw, parse_push, parse_pushw, parse_storew, AssemblyError,
        Operation, Token,
    };

    // TESTS FOR PUSHING VALUES ONTO THE STACK (PUSH)
    // ============================================================================================

    #[test]
    fn pushw_invalid() {
        test_parsew_base("pushw", "pushw.local|mem");
    }

    // TESTS FOR REMOVING VALUES FROM THE STACK (POP)
    // ============================================================================================

    #[test]
    fn popw_invalid() {
        test_parsew_base("popw", "popw.local|mem");
    }

    // TESTS FOR OVERWRITING VALUES ON THE STACK (LOAD)
    // ============================================================================================

    #[test]
    fn loadw_invalid() {
        test_parsew_base("loadw", "loadw.adv|local|mem");
    }

    // TESTS FOR SAVING STACK VALUES WITHOUT REMOVING THEM (STORE)
    // ============================================================================================

    #[test]
    fn storew_invalid() {
        test_parsew_base("storew", "storew.local|mem");
    }

    // TEST HELPERS
    // ============================================================================================

    /// Test that the core part of an instruction for an operation on a word is properly formed.
    /// It can be used to test operation and variant validity for `pushw`, `popw`, `loadw`, and
    /// `storew`.
    fn test_parsew_base(base_op: &str, expected_token: &str) {
        let num_proc_locals = 0;

        // fails when required variants are invalid or missing or the wrong operation is provided
        let pos = 0;

        // missing variant
        let op_no_variant = Token::new(base_op, pos);
        let expected = AssemblyError::invalid_op(&op_no_variant);
        assert_eq!(
            get_parsing_error(base_op, &op_no_variant, num_proc_locals),
            expected
        );

        // invalid variant
        let op_str = format!("{}.invalid", base_op);
        let op_invalid = Token::new(&op_str, pos);
        let expected = AssemblyError::unexpected_token(&op_invalid, expected_token);
        assert_eq!(
            get_parsing_error(base_op, &op_invalid, num_proc_locals),
            expected
        );

        // wrong operation passed to parsing function
        let op_mismatch = Token::new("none.mem", pos);
        let expected = AssemblyError::unexpected_token(&op_mismatch, expected_token);
        assert_eq!(
            get_parsing_error(base_op, &op_mismatch, num_proc_locals),
            expected
        )
    }

    /// Attempts to parse the specified operation, which is expected to be invalid, and returns the
    /// parsing error.
    pub fn get_parsing_error(
        base_op: &str,
        invalid_op: &Token,
        num_proc_locals: u32,
    ) -> AssemblyError {
        let mut span_ops: Vec<Operation> = Vec::new();

        match base_op {
            "push" => parse_push(&mut span_ops, invalid_op, num_proc_locals).unwrap_err(),
            "pushw" => parse_pushw(&mut span_ops, invalid_op, num_proc_locals).unwrap_err(),
            "pop" => parse_pop(&mut span_ops, invalid_op, num_proc_locals).unwrap_err(),
            "popw" => parse_popw(&mut span_ops, invalid_op, num_proc_locals).unwrap_err(),
            "loadw" => parse_loadw(&mut span_ops, invalid_op, num_proc_locals).unwrap_err(),
            "storew" => parse_storew(&mut span_ops, invalid_op, num_proc_locals).unwrap_err(),
            _ => panic!(),
        }
    }
}
