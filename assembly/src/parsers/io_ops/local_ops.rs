use super::{
    parse_u32_param, push_value, validate_operation, AssemblyError, Felt, Operation, Token, Vec,
};
use vm_core::utils::PushMany;

// LOCAL MEMORY FOR PROCEDURE VARIABLES
// ================================================================================================

/// Pushes the first element of the word at the specified local procedure memory index onto the
/// stack.
pub fn parse_push_local(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    num_proc_locals: u32,
) -> Result<(), AssemblyError> {
    validate_operation!(op, "push.local", 1);

    parse_read_local(span_ops, op, num_proc_locals, false)?;

    span_ops.push_many(Operation::Drop, 3);

    Ok(())
}

/// Pops the top element off the stack and saves it at the specified local procedure memory index as
/// [top_element, 0, 0, 0].
pub fn parse_pop_local(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    num_proc_locals: u32,
) -> Result<(), AssemblyError> {
    validate_operation!(op, "pop.local", 1);

    // pad to word length before calling STOREW
    span_ops.push_many(Operation::Pad, 3);

    parse_write_local(span_ops, op, num_proc_locals, false)
}

/// Translates the `pushw.local.i` and `loadw.local.i` assembly ops to the system's `LOADW` memory
/// read operation. When `overwrite_stack_top` is true, values should overwrite the top of the stack
/// (as required by `loadw`). When `overwrite_stack_top` is false, values should be pushed onto the
/// stack, leaving the rest of it unchanged (as required by `pushw`). This is achieved by first
/// using `PAD` to make space for 4 new elements.
///
/// # Errors
///
/// This function expects a memory read assembly operation that has already been validated, except
/// for its parameter. If called without validation, it could yield incorrect results or return an
/// `AssemblyError`. It will also return an `AssemblyError` if the op does not have an index
/// parameter specified.
pub fn parse_read_local(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    num_proc_locals: u32,
    overwrite_stack_top: bool,
) -> Result<(), AssemblyError> {
    validate_operation!(@only_params op, "pushw|loadw.local", 1);

    if !overwrite_stack_top {
        // make space for the new elements
        span_ops.push_many(Operation::Pad, 4);
    }

    push_local_addr(span_ops, op, num_proc_locals)?;
    span_ops.push(Operation::MLoadW);

    Ok(())
}

/// Translates the `popw.local.i` and `storew.local.i` assembly ops to the system's `STOREW` memory
/// write operation. When `retain_stack_top` is true, values should be left on the stack after being
/// written to memory (as required by `storew`). When `retain_stack_top` is false, values should be
/// dropped from the stack after being written (as required by `popw`).
///
/// # Errors
///
/// This function expects a memory write assembly operation that has already been validated, except
/// for its parameter. If called without validation, it could yield incorrect results or return an
/// `AssemblyError`. It will also return an `AssemblyError` if the op does not have an index
/// parameter specified.
pub fn parse_write_local(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    num_proc_locals: u32,
    retain_stack_top: bool,
) -> Result<(), AssemblyError> {
    validate_operation!(@only_params op, "popw|storew.local", 1);

    push_local_addr(span_ops, op, num_proc_locals)?;
    span_ops.push(Operation::MStoreW);

    if !retain_stack_top {
        span_ops.push_many(Operation::Drop, 4);
    }

    Ok(())
}

/// Parses a provided local memory index and pushes the corresponding absolute memory location onto
/// the stack.
///
/// # Errors
///
/// This function will return an `AssemblyError` if the index parameter is greater than the number
/// of locals declared by the procedure.
fn push_local_addr(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    num_proc_locals: u32,
) -> Result<(), AssemblyError> {
    if num_proc_locals == 0 {
        // if no procedure locals were declared, then no local mem ops are allowed
        return Err(AssemblyError::invalid_op_with_reason(
            op,
            "no procedure locals were declared",
        ));
    }

    // parse the provided local memory index
    let index = parse_u32_param(op, 2, 0, num_proc_locals - 1)?;

    // put the absolute memory address on the stack
    // negate the value to use it as an offset from the fmp
    // since the fmp value was incremented when locals were allocated
    push_value(span_ops, -Felt::new(index as u64));
    span_ops.push(Operation::FmpAdd);

    Ok(())
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{super::tests::get_parsing_error, AssemblyError, Token};

    // TESTS FOR PUSHING VALUES ONTO THE STACK (PUSH)
    // ============================================================================================

    #[test]
    fn push_local_invalid() {
        test_parse_local("push");
    }

    #[test]
    fn pushw_local_invalid() {
        test_parse_local("pushw");
    }

    // TESTS FOR REMOVING VALUES FROM THE STACK (POP)
    // ============================================================================================

    #[test]
    fn pop_local_invalid() {
        test_parse_local("pop");
    }

    #[test]
    fn popw_local_invalid() {
        test_parse_local("popw");
    }

    // TESTS FOR OVERWRITING VALUES ON THE STACK (LOAD)
    // ============================================================================================

    #[test]
    fn loadw_local_invalid() {
        test_parse_local("loadw");
    }

    // TESTS FOR SAVING STACK VALUES WITHOUT REMOVING THEM (STORE)
    // ============================================================================================

    #[test]
    fn storew_local_invalid() {
        test_parse_local("storew");
    }

    // TEST HELPERS
    // ============================================================================================

    /// Test that an instruction for a local memory operation is properly formed. It can be used to
    /// test parameter inputs for pushw.mem, popw.mem, loadw.mem, and storew.mem.
    fn test_parse_local(base_op: &str) {
        let num_proc_locals = 1;

        // fails when immediate values to a {push|pushw|pop|popw|loadw|storew}.local.i operation are
        // invalid or missing
        let pos = 0;

        // insufficient values provided
        let op_str = format!("{}.local", base_op);
        let op_val_missing = Token::new(&op_str, pos);
        let expected = AssemblyError::missing_param(&op_val_missing);
        assert_eq!(
            get_parsing_error(base_op, &op_val_missing, num_proc_locals),
            expected
        );

        // invalid value provided to local variant
        let op_str = format!("{}.local.abc", base_op);
        let op_val_invalid = Token::new(&op_str, pos);
        let expected = AssemblyError::invalid_param(&op_val_invalid, 2);
        assert_eq!(
            get_parsing_error(base_op, &op_val_invalid, num_proc_locals),
            expected
        ); // no procedure locals declared
        let op_str = format!("{}.local.{}", base_op, 1);
        let op_no_locals = Token::new(&op_str, pos);
        let expected = AssemblyError::invalid_op_with_reason(
            &op_no_locals,
            "no procedure locals were declared",
        );
        assert_eq!(get_parsing_error(base_op, &op_no_locals, 0), expected);

        // provided local index is outside of the declared bounds of the procedure locals
        let op_str = format!("{}.local.{}", base_op, num_proc_locals);
        let op_val_invalid = Token::new(&op_str, pos);
        let expected = AssemblyError::invalid_param_with_reason(
            &op_val_invalid,
            2,
            format!(
                "parameter value must be greater than or equal to 0 and less than or equal to {}",
                num_proc_locals - 1
            )
            .as_str(),
        );
        assert_eq!(
            get_parsing_error(base_op, &op_val_invalid, num_proc_locals),
            expected
        );

        // extra value provided to local variant
        let op_str = format!("{}.local.0.1", base_op);
        let op_extra_val = Token::new(&op_str, pos);
        let expected = AssemblyError::extra_param(&op_extra_val);
        assert_eq!(
            get_parsing_error(base_op, &op_extra_val, num_proc_locals),
            expected
        );
    }
}
