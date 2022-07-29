use super::{parse_u32_param, validate_operation, AssemblyError, Operation, Token, Vec};
use vm_core::utils::PushMany;

// CONSTANTS
// ================================================================================================

/// The maximum number of elements that can be read from the advice tape in a single `push`
/// operation.
const ADVICE_READ_LIMIT: u32 = 16;

// NON-DETERMINISTIC (ADVICE) INPUTS
// ================================================================================================

/// Appends the number of `READ` operations specified by the operation's immediate value
/// to the span block. This pushes the specified number of items from the advice tape onto the
/// stack. It limits the number of items that can be read from the advice tape at a time to 16.
///
/// # Errors
///
/// Returns an `AssemblyError` if the instruction is invalid, malformed, missing a required
/// parameter, or does not match the expected operation. Returns an `invalid_param` `AssemblyError`
/// if the parameter for `push.adv` is not a decimal value.
pub fn parse_push_adv(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    validate_operation!(op, "push.adv", 1);

    // parse and validate the parameter as the number of items to read from the advice tape
    // it must be between 1 and ADVICE_READ_LIMIT, inclusive, since adv.push.0 is a no-op
    let n = parse_u32_param(op, 2, 1, ADVICE_READ_LIMIT)?;

    // read n items from the advice tape and push then onto the stack
    span_ops.push_many(Operation::Read, n as usize);

    Ok(())
}

/// Reads a word from the advice tape and overwrites the top 4 elements of the stack with it. After
/// validation, this operation uses the ```READW``` machine operation directly.
pub fn parse_loadw_adv(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    validate_operation!(op, "loadw.adv", 0);

    // load a word from the advice tape
    span_ops.push(Operation::ReadW);

    Ok(())
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{
        super::{parse_loadw, parse_push},
        AssemblyError, Operation, Token, ADVICE_READ_LIMIT,
    };

    // TESTS FOR PUSHING VALUES ONTO THE STACK (PUSH)
    // ============================================================================================

    #[test]
    fn push_adv() {
        let num_proc_locals = 0;

        // remove n items from the advice tape and push them onto the stack
        let mut span_ops: Vec<Operation> = Vec::new();
        let op = Token::new("push.adv.4", 0);
        let expected = vec![Operation::Read; 4];

        parse_push(&mut span_ops, &op, num_proc_locals).expect("Failed to parse push.adv.4");
        assert_eq!(span_ops, expected);
    }

    #[test]
    fn push_adv_invalid() {
        let num_proc_locals = 0;

        // fails when the instruction is malformed or unrecognized
        let mut span_ops: Vec<Operation> = Vec::new();
        let pos = 0;

        // missing value
        let op_no_val = Token::new("push.adv", pos);
        let expected = AssemblyError::missing_param(&op_no_val);
        assert_eq!(
            parse_push(&mut span_ops, &op_no_val, num_proc_locals).unwrap_err(),
            expected
        );

        // extra value to push
        let op_extra_val = Token::new("push.adv.2.2", pos);
        let expected = AssemblyError::extra_param(&op_extra_val);
        assert_eq!(
            parse_push(&mut span_ops, &op_extra_val, num_proc_locals).unwrap_err(),
            expected
        );

        // invalid value - char
        let op_invalid_char = Token::new("push.adv.a", pos);
        let expected = AssemblyError::invalid_param(&op_invalid_char, 2);
        assert_eq!(
            parse_push(&mut span_ops, &op_invalid_char, num_proc_locals).unwrap_err(),
            expected
        );

        // invalid value - hexadecimal
        let op_invalid_hex = Token::new("push.adv.0x10", pos);
        let expected = AssemblyError::invalid_param(&op_invalid_hex, 2);
        assert_eq!(
            parse_push(&mut span_ops, &op_invalid_hex, num_proc_locals).unwrap_err(),
            expected
        );

        // parameter out of bounds
        let reason = format!(
            "parameter value must be greater than or equal to {} and less than or equal to {}",
            1, ADVICE_READ_LIMIT
        );
        // less than lower bound
        let op_lower_bound = Token::new("push.adv.0", pos);
        let expected = AssemblyError::invalid_param_with_reason(&op_lower_bound, 2, &reason);
        assert_eq!(
            parse_push(&mut span_ops, &op_lower_bound, num_proc_locals).unwrap_err(),
            expected
        );

        // greater than upper bound
        let inst_str = format!("push.adv.{}", ADVICE_READ_LIMIT + 1);
        let op_upper_bound = Token::new(&inst_str, pos);
        let expected = AssemblyError::invalid_param_with_reason(&op_upper_bound, 2, &reason);
        assert_eq!(
            parse_push(&mut span_ops, &op_upper_bound, num_proc_locals).unwrap_err(),
            expected
        );
    }

    // TESTS FOR OVERWRITING VALUES ON THE STACK (LOAD)
    // ============================================================================================
    #[test]
    fn loadw_adv() {
        let num_proc_locals = 0;

        // replace the top 4 elements of the stack with 4 elements from the advice tape
        let mut span_ops: Vec<Operation> = Vec::new();
        let op = Token::new("loadw.adv", 0);
        let expected = vec![Operation::ReadW];

        parse_loadw(&mut span_ops, &op, num_proc_locals).expect("Failed to parse loadw.adv");
        assert_eq!(span_ops, expected);
    }

    #[test]
    fn loadw_adv_invalid() {
        let num_proc_locals = 0;

        // fails when the instruction is malformed or unrecognized
        let mut span_ops: Vec<Operation> = Vec::new();
        let pos = 0;

        // extra value to loadw
        let op_extra_val = Token::new("loadw.adv.0", pos);
        let expected = AssemblyError::extra_param(&op_extra_val);
        assert_eq!(
            parse_loadw(&mut span_ops, &op_extra_val, num_proc_locals).unwrap_err(),
            expected
        );
    }
}
