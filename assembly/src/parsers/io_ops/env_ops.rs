use super::{
    parse_u32_param, push_value, validate_operation, AssemblyError, Felt, Operation, Token, Vec,
};

// ENVIRONMENT INPUTS
// ================================================================================================

/// Appends machine operations to the current span block according to the requested environment
/// assembly instruction.
///
/// - `push.env.locaddr.i` pushes the absolute address of the local variable at index `i` onto the
/// stack.
/// - `push.env.sdepth` pushes the current depth of the stack onto the top of the stack, which is
/// handled directly by the `SDEPTH` operation.
///
/// # Errors
///
/// This function expects a valid assembly environment op that specifies the environment input to
/// be handled. It will return an error if the assembly instruction is malformed or the environment
/// input is unrecognized.
pub fn parse_push_env(
    span_ops: &mut Vec<Operation>,
    op: &Token,
    num_proc_locals: u32,
) -> Result<(), AssemblyError> {
    validate_operation!(op, "push.env.locaddr|sdepth");

    // update the span block
    match op.parts()[2] {
        "locaddr" => {
            if num_proc_locals == 0 {
                return Err(AssemblyError::invalid_op_with_reason(
                    op,
                    "no procedure locals available in current context",
                ));
            }
            validate_operation!(@only_params op, "push.env.locaddr", 1);
            let index = parse_u32_param(op, 3, 0, num_proc_locals - 1)?;

            push_value(span_ops, -Felt::new(index as u64));
            span_ops.push(Operation::FmpAdd);
        }
        "sdepth" => {
            validate_operation!(@only_params op, "push.env.sdepth", 0);
            span_ops.push(Operation::SDepth);
        }
        _ => return Err(AssemblyError::invalid_op(op)),
    }

    Ok(())
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{
        super::tests::get_parsing_error,
        super::{super::FieldElement, parse_push, AssemblyError, Felt},
        Operation, Token,
    };

    // TESTS FOR PUSHING VALUES ONTO THE STACK (PUSH)
    // ============================================================================================

    #[test]
    fn push_env_sdepth() {
        let num_proc_locals = 0;

        // --- pushes the current depth of the stack onto the top of the stack --------------------
        let mut span_ops = vec![Operation::Push(Felt::ONE); 8];
        let op = Token::new("push.env.sdepth", 0);
        let mut expected = span_ops.clone();
        expected.push(Operation::SDepth);

        parse_push(&mut span_ops, &op, num_proc_locals)
            .expect("Failed to parse push.env.sdepth with empty stack");
        assert_eq!(span_ops, expected);
    }

    #[test]
    fn push_env_locaddr() {
        let asm_op = "push.env.locaddr";
        let num_proc_locals = 2;
        let mut span_ops: Vec<Operation> = Vec::new();

        let op_str = format!("{}.{}", asm_op, 1);
        let op = Token::new(&op_str, 0);
        assert!(parse_push(&mut span_ops, &op, num_proc_locals).is_ok());
    }

    #[test]
    fn push_env_invalid() {
        let num_proc_locals = 0;

        // --- fails when env op variant is invalid or missing or has too many immediate values ---
        let mut span_ops: Vec<Operation> = Vec::new();
        let pos = 0;

        // --- missing env var --------------------------------------------------------------------
        let op_no_val = Token::new("push.env", pos);
        let expected = AssemblyError::invalid_op(&op_no_val);
        assert_eq!(
            parse_push(&mut span_ops, &op_no_val, num_proc_locals).unwrap_err(),
            expected
        );

        // --- invalid env var --------------------------------------------------------------------
        let op_val_invalid = Token::new("push.env.invalid", pos);
        let expected = AssemblyError::unexpected_token(&op_val_invalid, "push.env.locaddr|sdepth");
        assert_eq!(
            parse_push(&mut span_ops, &op_val_invalid, num_proc_locals).unwrap_err(),
            expected
        );

        // --- extra value ------------------------------------------------------------------------
        let op_extra_val = Token::new("push.env.sdepth.0", pos);
        let expected = AssemblyError::extra_param(&op_extra_val);
        assert_eq!(
            parse_push(&mut span_ops, &op_extra_val, num_proc_locals).unwrap_err(),
            expected
        );
    }

    #[test]
    fn push_env_sdepth_invalid() {
        let num_proc_locals = 0;

        // fails when env op variant is invalid or missing or has too many immediate values
        let mut span_ops: Vec<Operation> = Vec::new();
        let pos = 0;

        // --- extra param ------------------------------------------------------------------------
        let op_extra_val = Token::new("push.env.sdepth.0", pos);
        let expected = AssemblyError::extra_param(&op_extra_val);
        assert_eq!(
            parse_push(&mut span_ops, &op_extra_val, num_proc_locals).unwrap_err(),
            expected
        );
    }

    #[test]
    fn push_env_locaddr_invalid() {
        let asm_op = "push.env.locaddr";
        let num_proc_locals = 2;
        let mut span_ops: Vec<Operation> = Vec::new();
        let pos = 0;

        // --- missing param ----------------------------------------------------------------------
        let op_missing_param = Token::new(asm_op, pos);
        let expected = AssemblyError::missing_param(&op_missing_param);
        assert_eq!(
            parse_push(&mut span_ops, &op_missing_param, num_proc_locals).unwrap_err(),
            expected
        );

        // --- provided local index is outside of the declared bounds of the procedure locals -----
        let op_str = format!("{}.{}", asm_op, 2);
        let op_val_invalid = Token::new(&op_str, pos);
        let expected = AssemblyError::invalid_param_with_reason(
            &op_val_invalid,
            3,
            format!(
                "parameter value must be greater than or equal to 0 and less than or equal to {}",
                num_proc_locals - 1
            )
            .as_str(),
        );
        assert_eq!(
            get_parsing_error("push", &op_val_invalid, num_proc_locals),
            expected
        );

        // --- no procedure locals in context -----------------------------------------------------
        let op_str = format!("{}.{}", asm_op, 1);
        let op_context_invalid = Token::new(&op_str, pos);
        let expected = AssemblyError::invalid_op_with_reason(
            &op_context_invalid,
            "no procedure locals available in current context",
        );
        assert_eq!(get_parsing_error("push", &op_context_invalid, 0), expected);
    }
}
