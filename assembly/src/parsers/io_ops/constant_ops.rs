use super::{
    parse_decimal_param, parse_element_param, parse_hex_param, push_value, validate_operation,
    AssemblyError, Felt, Operation, Token, Vec,
};

// CONSTANTS
// ================================================================================================

/// The maximum number of constant inputs allowed by `push` operation.
const MAX_CONST_INPUTS: usize = 16;

/// The required length of the hexadecimal representation for an input value when more than one hex
/// input is provided to `push` without period separators.
const HEX_CHUNK_SIZE: usize = 16;

// CONSTANT INPUTS
// ================================================================================================

/// Appends `PUSH` operations to the span block to push one or more provided constant values onto
/// the stack, up to a maximum of 16 values.
///
/// Constant values may be specified in one of 2 formats:
/// 1. A series of 1-16 valid field elements in decimal or hexadecimal representation separated by
///    periods, e.g. push.0x1234.0xabcd
/// 2. A hexadecimal string without period separators that represents a series of 1-16 elements
///    where the total number of specified bytes is a multiple of 8, e.g.
///    push.0x0000000000001234000000000000abcd
///
/// In cases when the immediate value is 0, `PUSH` operation is replaced with `PAD`. Also, in cases
/// when immediate value is 1, `PUSH` operation is replaced with `PAD INCR` because in most cases
/// this will be more efficient than doing a `PUSH`.
///
/// # Errors
///
/// It will return an error if no immediate value is provided or if any of parameter formats are
/// invalid. It will also return an error if the op token is malformed or doesn't match the expected
/// instruction.
pub fn parse_push_constant(span_ops: &mut Vec<Operation>, op: &Token) -> Result<(), AssemblyError> {
    validate_operation!(op, "push", 1..MAX_CONST_INPUTS);

    let param_idx = 1;
    let param_count = op.num_parts() - param_idx;
    // for multiple input parameters, parse & push each one onto the stack in order, then return
    if param_count > 1 {
        for param_idx in param_idx..=param_count {
            let value = parse_element_param(op, param_idx)?;
            push_value(span_ops, value);
        }
        return Ok(());
    }

    // for a single input, there could be one value or there could be a series of many hexadecimal
    // values without separators
    let param_str = op.parts()[param_idx];
    if let Some(param_str) = param_str.strip_prefix("0x") {
        // parse 1 or more hexadecimal values
        let values = parse_hex_params(op, param_idx, param_str)?;
        // push each value onto the stack in order
        for &value in values.iter() {
            push_value(span_ops, value);
        }
    } else {
        // parse 1 decimal value and push it onto the stack
        let value = parse_decimal_param(op, param_idx, param_str)?;
        push_value(span_ops, value);
    }

    Ok(())
}

/// Parses a hexadecimal string into a vector of 1 or more field elements, up to 16 total. If more
/// than one value is specified, then the total number of specified bytes must be a multiple of 8.
fn parse_hex_params(
    op: &Token,
    param_idx: usize,
    param_str: &str,
) -> Result<Vec<Felt>, AssemblyError> {
    // handle error cases where the hex string is poorly formed
    let is_single_element = if param_str.len() <= HEX_CHUNK_SIZE {
        if param_str.len() % 2 != 0 {
            // parameter string is not a valid hex representation
            return Err(AssemblyError::invalid_param(op, param_idx));
        }
        true
    } else {
        if param_str.len() % HEX_CHUNK_SIZE != 0 {
            // hex string doesn't contain a valid number of bytes
            return Err(AssemblyError::invalid_param(op, param_idx));
        } else if param_str.len() > HEX_CHUNK_SIZE * MAX_CONST_INPUTS {
            // hex string contains more than the maximum number of inputs
            return Err(AssemblyError::extra_param(op));
        }
        false
    };

    // parse the hex string into one or more valid field elements
    if is_single_element {
        // parse a single element in hex representation
        let parsed_param = parse_hex_param(op, param_idx, param_str)?;
        Ok(vec![parsed_param])
    } else {
        // iterate over the multi-value hex string and parse each 8-byte chunk into a valid element
        (0..param_str.len())
            .step_by(HEX_CHUNK_SIZE)
            .map(|i| parse_hex_param(op, param_idx, &param_str[i..i + HEX_CHUNK_SIZE]))
            .collect()
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::{super::parse_push, AssemblyError, Felt, Operation, Token};

    // TESTS FOR PUSHING VALUES ONTO THE STACK (PUSH)
    // ============================================================================================

    #[test]
    fn push_one() {
        let num_proc_locals = 0;

        let mut span_ops: Vec<Operation> = Vec::new();
        let op_0 = Token::new("push.0", 0);
        let op_1 = Token::new("push.1", 0);
        let op_dec = Token::new("push.135", 0);
        let op_hex = Token::new("push.0x7b", 0);
        let expected = vec![
            Operation::Pad,
            Operation::Pad,
            Operation::Incr,
            Operation::Push(Felt::new(135)),
            Operation::Push(Felt::new(123)),
        ];

        parse_push(&mut span_ops, &op_0, num_proc_locals).expect("Failed to parse push.0");
        parse_push(&mut span_ops, &op_1, num_proc_locals).expect("Failed to parse push.1");
        parse_push(&mut span_ops, &op_dec, num_proc_locals)
            .expect("Failed to parse push of decimal element 123");
        parse_push(&mut span_ops, &op_hex, num_proc_locals)
            .expect("Failed to parse push of hex element 0x7b");

        assert_eq!(span_ops, expected);
    }

    #[test]
    fn push_many() {
        let num_proc_locals = 0;

        // --- push 4 decimal values --------------------------------------------------------------
        let mut span_ops: Vec<Operation> = Vec::new();
        let op_4_dec = Token::new("push.4.5.6.7", 0);
        let mut expected = Vec::with_capacity(4);
        for a in 4..8 {
            expected.push(Operation::Push(Felt::new(a)));
        }
        parse_push(&mut span_ops, &op_4_dec, num_proc_locals)
            .expect("Failed to parse push.4.5.6.7");
        assert_eq!(span_ops, expected);

        // --- push the maximum number of decimal values (16) -------------------------------------
        let mut span_ops: Vec<Operation> = Vec::new();
        let op_16_dec = Token::new("push.16.17.18.19.20.21.22.23.24.25.26.27.28.29.30.31", 0);
        let mut expected = Vec::with_capacity(16);
        for a in 16..32 {
            expected.push(Operation::Push(Felt::new(a)));
        }
        parse_push(&mut span_ops, &op_16_dec, num_proc_locals)
            .expect("Failed to parse push.16.17.18.19.20.21.22.23.24.25.26.27.28.29.30.31");
        assert_eq!(span_ops, expected);

        // --- push hexadecimal values with period separators between values ----------------------
        let mut span_ops: Vec<Operation> = Vec::new();
        let op_5_hex = Token::new("push.0xA.0x64.0x3E8.0x2710.0x186A0", 0);
        let mut expected = Vec::with_capacity(5);
        for i in 1..=5 {
            expected.push(Operation::Push(Felt::new(10_u64.pow(i))));
        }
        parse_push(&mut span_ops, &op_5_hex, num_proc_locals)
            .expect("Failed to parse push.0xA.0x64.0x3EB.0x2710.0x186A0");
        assert_eq!(span_ops, expected);

        // --- push a mixture of decimal and single-element hexadecimal values --------------------
        let mut span_ops: Vec<Operation> = Vec::new();
        let op_8_dec_hex = Token::new("push.2.4.8.0x10.0x20.0x40.128.0x100", 0);
        let mut expected = Vec::with_capacity(8);
        for i in 1_u32..=8 {
            expected.push(Operation::Push(Felt::new(2_u64.pow(i))));
        }
        parse_push(&mut span_ops, &op_8_dec_hex, num_proc_locals)
            .expect("Failed to parse push.2.4.8.0x10.0x20.0x40.128.0x100");
        assert_eq!(span_ops, expected);
    }

    #[test]
    fn push_without_separator() {
        let num_proc_locals = 0;
        // --- push hexadecimal values with no period separators ----------------------------------
        let mut span_ops: Vec<Operation> = Vec::new();
        let mut expected: Vec<Operation> = Vec::new();
        let op_sep = Token::new("push.0x1234.0xabcd", 0);
        let op_no_sep = Token::new("push.0x0000000000001234000000000000abcd", 0);
        parse_push(&mut expected, &op_sep, num_proc_locals)
            .expect("Failed to parse push.0x1234.0xabcd");
        parse_push(&mut span_ops, &op_no_sep, num_proc_locals)
            .expect("Failed to parse push.0x0000000000001234000000000000abcd");
        assert_eq!(span_ops, expected);

        // --- push the maximum number of hexadecimal values without separators (16) --------------
        let mut span_ops: Vec<Operation> = Vec::new();
        let mut expected: Vec<Operation> = Vec::new();
        let op_16_dec = Token::new("push.0.1.2.3.4.5.6.7.8.9.10.11.12.13.14.15", 0);
        let op_16_no_sep = Token::new("push.0x0000000000000000000000000000000100000000000000020000000000000003000000000000000400000000000000050000000000000006000000000000000700000000000000080000000000000009000000000000000A000000000000000B000000000000000C000000000000000D000000000000000E000000000000000F", 0);
        parse_push(&mut expected, &op_16_dec, num_proc_locals)
            .expect("Failed to parse push.0.1.2.3.4.5.6.7.8.9.10.11.12.13.14.15");
        parse_push(&mut span_ops, &op_16_no_sep,num_proc_locals)
            .expect("Failed to parse push.0x0000000000000000000000000000000100000000000000020000000000000003000000000000000400000000000000050000000000000006000000000000000700000000000000080000000000000009000000000000000A000000000000000B000000000000000C000000000000000D000000000000000E000000000000000F");
        assert_eq!(span_ops, expected);
    }

    #[test]
    fn push_invalid() {
        let num_proc_locals = 0;

        // fails when immediate value is invalid or missing
        let mut span_ops: Vec<Operation> = Vec::new();
        let pos = 0;

        // missing value or variant
        let op_no_val = Token::new("push", pos);
        let expected = AssemblyError::invalid_op(&op_no_val);
        assert_eq!(
            parse_push(&mut span_ops, &op_no_val, num_proc_locals).unwrap_err(),
            expected
        );

        // invalid value
        let op_val_invalid = Token::new("push.abc", pos);
        let expected = AssemblyError::invalid_param(&op_val_invalid, 1);
        assert_eq!(
            parse_push(&mut span_ops, &op_val_invalid, num_proc_locals).unwrap_err(),
            expected
        );

        // --- separators for single values cannot be combined with multi-element hex inputs ------
        let mut span_ops: Vec<Operation> = Vec::new();
        let op_mixed_sep = Token::new("push.4.5.0x0000000000001234000000000000abcd", 0);
        let expected = AssemblyError::invalid_param(&op_mixed_sep, 3);
        assert_eq!(
            parse_push(&mut span_ops, &op_mixed_sep, num_proc_locals).unwrap_err(),
            expected
        );

        let mut span_ops: Vec<Operation> = Vec::new();
        let op_mixed_sep = Token::new("push.0x0000000000001234000000000000abcd.4.5", 0);
        let expected = AssemblyError::invalid_param(&op_mixed_sep, 1);
        assert_eq!(
            parse_push(&mut span_ops, &op_mixed_sep, num_proc_locals).unwrap_err(),
            expected
        );

        // extra value
        let op_extra_val = Token::new("push.0.1.2.3.4.5.6.7.8.9.10.11.12.13.14.15.16", pos);
        let expected = AssemblyError::extra_param(&op_extra_val);
        assert_eq!(
            parse_push(&mut span_ops, &op_extra_val, num_proc_locals).unwrap_err(),
            expected
        );

        // wrong operation passed to parsing function
        let op_mismatch = Token::new("pushw.0", pos);
        let expected = AssemblyError::unexpected_token(
            &op_mismatch,
            "push.{adv.n|env.var|local.i|mem|mem.a|a|a.b|a.b.c...}",
        );
        assert_eq!(
            parse_push(&mut span_ops, &op_mismatch, num_proc_locals).unwrap_err(),
            expected
        )
    }

    #[test]
    fn push_invalid_hex() {
        let num_proc_locals = 0;

        // --- no separators, and total number of bytes is not a multiple of 8 --------------------
        let mut span_ops: Vec<Operation> = Vec::new();
        let op_bad_hex = Token::new("push.0x00000000000012340000abcd", 0);
        let expected = AssemblyError::invalid_param(&op_bad_hex, 1);
        assert_eq!(
            parse_push(&mut span_ops, &op_bad_hex, num_proc_locals).unwrap_err(),
            expected
        );

        // --- some period separators, but total number of bytes is not a multiple of 8 --------------------
        let mut span_ops: Vec<Operation> = Vec::new();
        let op_bad_hex = Token::new("push.0x00001234000000000000abcd.0x5678.0x9ef0", 0);
        let expected = AssemblyError::invalid_param(&op_bad_hex, 1);
        assert_eq!(
            parse_push(&mut span_ops, &op_bad_hex, num_proc_locals).unwrap_err(),
            expected
        );

        // --- too many values provided in hex format without separators --------------
        let mut span_ops: Vec<Operation> = Vec::new();
        let op_extra_val = Token::new("push.0x0000000000000000000000000000000100000000000000020000000000000003000000000000000400000000000000050000000000000006000000000000000700000000000000080000000000000009000000000000000A000000000000000B000000000000000C000000000000000D000000000000000E000000000000000F0000000000000010", 0);
        let expected = AssemblyError::extra_param(&op_extra_val);
        assert_eq!(
            parse_push(&mut span_ops, &op_extra_val, num_proc_locals).unwrap_err(),
            expected
        );
    }
}
